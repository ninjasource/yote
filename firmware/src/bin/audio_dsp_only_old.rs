#![no_std]
#![no_main]

use cs47l63::driver;
use yote::{
    hw_dsp::{dsp, shared_bus::SharedBus},
    play_state::PlayState,
    wave::{self, NUM_SAMPLES},
};

use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::Pin;
use embassy_nrf::gpiote::{AnyChannel, Channel};
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

use {defmt_rtt as _, panic_probe as _};

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
    // NB: do not remove this as it is very important to set P0_11 high so that the spi bus can be used with the codec cs pin P0_17
    let _cs_sdcard = Output::new(AnyPin::from(p.P0_11), Level::High, OutputDrive::Standard); // shared bus (must remain high)

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
    let mut hw_codec_sel_out =
        Output::new(AnyPin::from(p.P0_21), Level::High, OutputDrive::Standard);
    let mut _spi_sel_in = Input::new(AnyPin::from(p.P0_22), Pull::None);
    let mut _pmic_iset_out = Output::new(AnyPin::from(p.P0_23), Level::Low, OutputDrive::HighDrive);
    let mut board_id_en_out =
        Output::new(AnyPin::from(p.P0_24), Level::Low, OutputDrive::HighDrive);
    let mut _board_id_in = Input::new(AnyPin::from(p.P0_27), Pull::None);

    // gpio buttons
    let btn1 = Input::new(p.P0_02.degrade(), Pull::Up);
    let btn2 = Input::new(p.P0_03.degrade(), Pull::Up);
    let btn3 = Input::new(AnyPin::from(p.P0_04), Pull::Up);
    let btn4 = Input::new(AnyPin::from(p.P0_06), Pull::Up);
    let btn5 = Input::new(AnyPin::from(p.P0_05), Pull::Up);
    let btn1_vol_down =
        InputChannel::new(p.GPIOTE_CH0.degrade(), btn1, InputChannelPolarity::HiToLo);
    let btn2_vol_up = InputChannel::new(p.GPIOTE_CH1.degrade(), btn2, InputChannelPolarity::HiToLo);
    let btn3_play = InputChannel::new(p.GPIOTE_CH2.degrade(), btn3, InputChannelPolarity::HiToLo);
    let btn4_mute = InputChannel::new(p.GPIOTE_CH3.degrade(), btn4, InputChannelPolarity::HiToLo);
    let btn5_tone = InputChannel::new(p.GPIOTE_CH4.degrade(), btn5, InputChannelPolarity::HiToLo);

    // gpio leds
    let _rgb1_red = Output::new(AnyPin::from(p.P0_07), Level::Low, OutputDrive::HighDrive);
    let _rgb1_green = Output::new(AnyPin::from(p.P0_25), Level::Low, OutputDrive::HighDrive);
    let _rgb1_blue = Output::new(AnyPin::from(p.P0_26), Level::Low, OutputDrive::HighDrive);
    let _rgb2_red = Output::new(AnyPin::from(p.P0_28), Level::Low, OutputDrive::HighDrive);
    let _rgb2_green = Output::new(AnyPin::from(p.P0_29), Level::Low, OutputDrive::HighDrive);
    let _rgb2_blue = Output::new(AnyPin::from(p.P0_30), Level::Low, OutputDrive::HighDrive);
    let _led1_blue = Output::new(AnyPin::from(p.P0_31), Level::Low, OutputDrive::HighDrive);
    let _led2_green = Output::new(AnyPin::from(p.P1_00), Level::Low, OutputDrive::HighDrive);
    let _led3_green = Output::new(AnyPin::from(p.P1_01), Level::Low, OutputDrive::HighDrive);

    // i2s sound bus for full duplex audio
    let master_clock: MasterClock = i2s::ApproxSampleRate::_11025.into();
    let sample_rate = master_clock.sample_rate();
    info!("Sample rate: {}", sample_rate);
    let mut config = i2s::Config::default();
    config.sample_width = i2s::SampleWidth::_16bit;
    config.channels = i2s::Channels::MonoLeft;
    let buffers_in = i2s::DoubleBuffering::<wave::Sample, NUM_SAMPLES>::new();
    let buffers_out = i2s::DoubleBuffering::<wave::Sample, NUM_SAMPLES>::new();
    let _stream = I2S::new_master(
        p.I2S0,
        Irqs,
        p.P0_12,
        p.P0_14,
        p.P0_16,
        master_clock,
        config,
    )
    .full_duplex(p.P0_15, p.P0_13, buffers_out, buffers_in);

    // wait for things to settle
    Timer::after(Duration::from_millis(100)).await;

    // I don't know what this does - does not seem to do anything
    board_id_en_out.set_high();

    // controls play / pause state messaging between tasks
    static PLAY_STATE: PlayState = PlayState::new();
    static TONE_PLAYING: AtomicBool = AtomicBool::new(false);

    // task for responding to irq events from dsp
    unwrap!(spawner.spawn(process_events(shared_bus, hw_codec_irq)));

    // task for responding to button press events
    unwrap!(spawner.spawn(process_buttons(
        shared_bus,
        btn1_vol_down,
        btn2_vol_up,
        btn3_play,
        btn4_mute,
        btn5_tone,
        &PLAY_STATE,
        &TONE_PLAYING
    )));

    if let Err(e) =
        audio_system_init(shared_bus, &mut hw_codec_sel_out, &mut hw_codec_reset_out).await
    {
        error!("Error initialising audio codec: {:?}", e);
        return;
    }

    info!("Ready");

    loop {
        Timer::after(Duration::from_millis(1000)).await;
        info!("Tick");
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

#[allow(clippy::too_many_arguments)]
#[embassy_executor::task(pool_size = 1)]
async fn process_buttons(
    shared_bus: &'static SharedBus,
    button1_vol_down: InputChannel<'static, AnyChannel, AnyPin>,
    button2_vol_up: InputChannel<'static, AnyChannel, AnyPin>,
    button3_play: InputChannel<'static, AnyChannel, AnyPin>,
    button4_mute: InputChannel<'static, AnyChannel, AnyPin>,
    button5_tone: InputChannel<'static, AnyChannel, AnyPin>,
    play_state: &'static PlayState,
    tone_playing: &'static AtomicBool,
) {
    const VOLUME_ADJUST_STEP_DB: i32 = 3;
    info!("[BTN_TASK] Waiting for buttons");

    let button1 = volume_button_handler(button1_vol_down, shared_bus, -VOLUME_ADJUST_STEP_DB);
    let button2 = volume_button_handler(button2_vol_up, shared_bus, VOLUME_ADJUST_STEP_DB);
    let button3 = async {
        loop {
            button3_play.wait().await;
            info!("[BTN_TASK] Play / Pause button pressed");
            play_state.toggle().await;
            debounce_button().await;
        }
    };
    let button4 = mute_button_handler(button4_mute, shared_bus);
    let button5 = async {
        let mut is_playing = false;
        loop {
            button5_tone.wait().await;
            is_playing = !is_playing;
            info!("[BTN_TASK] Tone button pressed: {}", is_playing);
            tone_playing.store(is_playing, Ordering::SeqCst);
            debounce_button().await;
        }
    };

    futures::join!(button1, button2, button3, button4, button5);
}

async fn volume_button_handler(
    volume_button: InputChannel<'static, AnyChannel, AnyPin>,
    shared_bus: &'static SharedBus,
    adjustment_db: i32,
) {
    loop {
        volume_button.wait().await;
        let mut bus = shared_bus.borrow().await;
        match dsp::volume_adjust(&mut bus, adjustment_db).await {
            Ok(level_db) => info!("[BTN_TASK] Volume set to {} dB", level_db),
            Err(e) => error!("[BTN_TASK] Error setting volume: {:?}", e),
        }

        debounce_button().await;
    }
}

async fn mute_button_handler(
    mute_button: InputChannel<'static, AnyChannel, AnyPin>,
    shared_bus: &'static SharedBus,
) {
    let mut mute = false;
    loop {
        mute_button.wait().await;
        let mut bus = shared_bus.borrow().await;

        mute = !mute;
        match dsp::volume_mute(&mut bus, mute).await {
            Ok(()) => {
                if mute {
                    info!("[BTN_TASK] Muted");
                } else {
                    info!("[BTN_TASK] Unmuted");
                }
            }
            Err(e) => error!("[BTN_TASK] Error handling mute button: {:?}", e),
        }

        debounce_button().await;
    }
}

async fn debounce_button() {
    Timer::after(Duration::from_millis(100)).await;
}

async fn audio_system_init(
    shared_bus: &SharedBus,
    hw_codec_sel_out: &mut Output<'_, AnyPin>,
    hw_codec_reset_out: &mut Output<'_, AnyPin>,
) -> Result<(), spim::Error> {
    // select the on-board HW codec
    hw_codec_sel_out.set_low();
    Timer::after(Duration::from_millis(2)).await;

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
