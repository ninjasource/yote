#![no_std]
#![no_main]

use cs47l63::driver;
use yote::{
    hw_dsp::{dsp, shared_bus::SharedBus},
    play_state::PlayState,
    wave::{self, Waveform, NUM_SAMPLES},
};

use core::mem;
use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::Pin;
use embassy_nrf::gpiote::{AnyChannel, Channel};
use embassy_nrf::i2s::FullDuplexStream;
use embassy_nrf::{
    bind_interrupts,
    gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull},
    gpiote::{InputChannel, InputChannelPolarity},
    i2s::{self, MasterClock, I2S},
    peripherals::{I2S0, SERIAL3},
    spim::{self, Frequency},
};
use embassy_time::{Duration, Timer};
use nrf5340_app_pac as pac;
use static_cell::StaticCell;

// use {defmt_rtt as _, panic_probe as _};
use defmt_rtt as _;

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    defmt::flush();

    // restart on crash
    cortex_m::peripheral::SCB::sys_reset();
}

bind_interrupts!(struct Irqs {
    SERIAL3 => spim::InterruptHandler<SERIAL3>;
    I2S0 => i2s::InterruptHandler<I2S0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Started");

    // change app core clock from 64mhz to 128mhz for improved performance
    let clock: pac::CLOCK_S = unsafe { mem::transmute(()) };
    clock.hfclkctrl.write(|w| w.hclk().div1());
    info!("Set app core to 128mhz");

    // enable flash cache for improved performance
    let cache: pac::CACHE_S = unsafe { mem::transmute(()) };
    cache.enable.write(|w| w.enable().enabled());
    info!("Enabled flash cache");

    // setup peripherals for nrf5340 audio dk board
    let p = embassy_nrf::init(Default::default());

    // spi setup
    let mut config = spim::Config::default();
    config.frequency = Frequency::M4;
    let spi = spim::Spim::new(p.SERIAL3, Irqs, p.P0_08, p.P0_10, p.P0_09, config);
    let cs_codec = Output::new(AnyPin::from(p.P0_17), Level::High, OutputDrive::Standard);

    // create an spi bus that can be shared between tasks
    let shared_bus = SharedBus::new(spi, cs_codec);
    static SHARED_BUS: StaticCell<SharedBus> = StaticCell::new();
    let shared_bus = &*SHARED_BUS.init(shared_bus);

    // gpio setup
    let _hw_codec_gpio_in = Input::new(AnyPin::from(p.P0_20), Pull::None);
    let irq_in = Input::new(AnyPin::from(p.P0_19), Pull::None);
    let hw_codec_irq =
        InputChannel::new(p.GPIOTE_CH5.degrade(), irq_in, InputChannelPolarity::LoToHi);
    let mut hw_codec_reset_out =
        Output::new(AnyPin::from(p.P0_18), Level::High, OutputDrive::Standard);
    let mut _pmic_iset_out = Output::new(AnyPin::from(p.P1_01), Level::Low, OutputDrive::HighDrive);

    // gpio buttons
    let btn = Input::new(p.P1_00.degrade(), Pull::Up);
    let btn_play_pause =
        InputChannel::new(p.GPIOTE_CH0.degrade(), btn, InputChannelPolarity::HiToLo);

    // gpio leds
    let _rgb1_red = Output::new(AnyPin::from(p.P0_04), Level::Low, OutputDrive::HighDrive);

    // i2s sound bus for full duplex audio
    let master_clock: MasterClock = i2s::ApproxSampleRate::_11025.into();
    let sample_rate = master_clock.sample_rate();
    info!("Sample rate: {}", sample_rate);
    let mut config = i2s::Config::default();
    config.sample_width = i2s::SampleWidth::_16bit;
    config.channels = i2s::Channels::MonoLeft;
    let buffers_in = i2s::DoubleBuffering::<wave::Sample, NUM_SAMPLES>::new();
    let buffers_out = i2s::DoubleBuffering::<wave::Sample, NUM_SAMPLES>::new();
    let mut stream = I2S::new_master(
        p.I2S0,
        Irqs,
        p.P0_12,
        p.P0_14,
        p.P0_16,
        master_clock,
        config,
    )
    .full_duplex(p.P0_15, p.P0_24, buffers_out, buffers_in);

    info!("Wait for things to settle");

    // wait for things to settle
    Timer::after(Duration::from_millis(500)).await;

    // controls play / pause state messaging between tasks
    static PLAY_STATE: PlayState = PlayState::new();

    // task for responding to irq events from dsp
    unwrap!(spawner.spawn(process_events(shared_bus, hw_codec_irq)));

    // task for responding to button press events
    unwrap!(spawner.spawn(process_buttons(btn_play_pause, &PLAY_STATE)));

    if let Err(e) = audio_system_init(shared_bus, &mut hw_codec_reset_out).await {
        error!("Error initialising audio codec: {:?}", e);
        return;
    }

    info!("Ready");

    // play audio tone
    if let Err(e) = play_audio(
        &PLAY_STATE,
        sample_rate,
        &mut stream,
        shared_bus,
        &mut hw_codec_reset_out,
    )
    .await
    {
        error!("Error playing audio: {:?}", e);
    }
}

async fn play_audio(
    play_state: &PlayState,
    sample_rate: u32,
    stream: &mut FullDuplexStream<'static, I2S0, i16, 2, 32>,
    shared_bus: &'static SharedBus,
    hw_codec_reset_out: &mut Output<'_, AnyPin>,
) -> Result<(), i2s::Error> {
    stream.start().await?;
    let mut waveform = Waveform::new(440.0, sample_rate as f32);

    let mut bus = shared_bus.borrow().await;
    match dsp::volume_adjust(&mut bus, 12).await {
        Ok(level_db) => info!("Volume set to {} dB", level_db),
        Err(e) => error!("Error setting volume: {:?}", e),
    }

    let mut counter = 0;
    // Note: this starts off paused, waiting for the user to press the Play / Pause button
    loop {
        if play_state.is_playing() {
            let (out_buf, in_buf) = stream.buffers();
            out_buf.copy_from_slice(in_buf);

            /*
            // copy mic input
            for (x_in, x_out) in in_buf.iter().zip(out_buf) {
                *x_out = *x_in * 5;
            }*/

            stream.send_and_receive().await?;

            counter += 1;

            if counter == 2000 {
                info!("Counter up");
                hw_codec_reset_out.set_low();
                loop {
                    Timer::after(Duration::from_secs(1)).await;
                    info!("Do nothing...");
                }
            }
        } else {
            // play silence
            info!("Playback paused");
            let (out_buf, _) = stream.buffers();
            waveform.zero(out_buf);
            stream.send_and_receive().await?;

            // wait for the signal that the play button has been pressed
            play_state.wait().await;
            info!("Playback started");
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn process_events(
    shared_bus: &'static SharedBus,
    hw_codec_irq: InputChannel<'static, AnyChannel, AnyPin>,
) {
    loop {
        info!("[EVT_TASK] Waiting for IRQ");
        hw_codec_irq.wait().await;
        info!("[EVT_TASK] IRQ triggered, handling events");

        let event_flags = {
            // only borrow the bus for as long as it takes to process the event handler
            let mut bus = shared_bus.borrow().await;
            driver::event_handler(&mut bus).await
        };

        match event_flags {
            Ok(event_flags) => info!(
                "[EVT_TASK] Completed handling IRQ triggered events: {}",
                event_flags
            ),
            Err(e) => error!("[EVT_TASK] Error handling IRQ triggered events: {:?}", e),
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn process_buttons(
    btn_play_pause: InputChannel<'static, AnyChannel, AnyPin>,
    play_state: &'static PlayState,
) {
    info!("[BTN_TASK] Waiting for buttons");

    loop {
        btn_play_pause.wait().await;
        info!("[BTN_TASK] Play / Pause button pressed");
        play_state.toggle().await;
        debounce_button().await;
    }
}

async fn debounce_button() {
    Timer::after(Duration::from_millis(100)).await;
}

async fn audio_system_init(
    shared_bus: &SharedBus,
    hw_codec_reset_out: &mut Output<'_, AnyPin>,
) -> Result<(), spim::Error> {
    // drive RESET low then high
    hw_codec_reset_out.set_low();
    Timer::after(Duration::from_millis(24)).await;
    hw_codec_reset_out.set_high();

    // reset
    dsp::reset(shared_bus).await?;
    info!("System init and reset complete");

    // configure codec
    dsp::default_conf_enable(shared_bus).await?;
    info!("HW codec configured for streaming");

    // enable equalizer
    //dsp::enable_equalizer(shared_bus).await?;
    //info!("Equalizer enabled");

    // enable compression
    //dsp::enable_compression(shared_bus).await?;
    //info!("Compression enabled");

    Ok(())
}
