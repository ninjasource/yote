#![no_std]
#![no_main]

use cs47l63::hw_interface::Bus;
use embassy_nrf::gpio::{Input, Pin, Pull};
use embassy_nrf::peripherals::P0_18;
use yote::hw_dsp::dsp;
use yote::hw_dsp::shared_bus::SpiBusInnerFixed;

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpiote::{Channel, InputChannelPolarity};
use embassy_nrf::{
    bind_interrupts,
    gpio::{AnyPin, Level, Output, OutputDrive},
    gpiote::InputChannel,
    peripherals::SERIAL3,
    spim::{self, Frequency},
};
use embassy_time::{Duration, Timer};

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    defmt::flush();

    // restart on crash
    cortex_m::peripheral::SCB::sys_reset();
}

bind_interrupts!(struct Irqs {
    SERIAL3 => spim::InterruptHandler<SERIAL3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Started");

    // setup peripherals for nrf5340 audio dk board
    let mut p = embassy_nrf::init(Default::default()); // gpio setup
    let mut hw_codec_reset_out = Output::new(&mut p.P0_18, Level::High, OutputDrive::Standard);
    let _cs_sdcard = Output::new(AnyPin::from(p.P0_11), Level::High, OutputDrive::Standard); // shared bus (must remain high)

    // gpio buttons
    let btn = Input::new(p.P0_04.degrade(), Pull::Up);
    let btn_1 = InputChannel::new(p.GPIOTE_CH0.degrade(), btn, InputChannelPolarity::HiToLo);

    info!("Wait for things to settle");

    // wait for things to settle
    Timer::after(Duration::from_millis(50)).await;
    info!("Ready");

    let mut is_playing = true;

    loop {
        info!("Resetting audio codec");
        hw_codec_reset_out.set_low();
        Timer::after(Duration::from_millis(24)).await;
        hw_codec_reset_out.set_high();

        if is_playing {
            info!("Initialising dsp");
            let mut config = spim::Config::default();
            config.frequency = Frequency::M4;
            let spi = spim::Spim::new(
                &mut p.SERIAL3,
                Irqs,
                &mut p.P0_08,
                &mut p.P0_10,
                &mut p.P0_09,
                config,
            );
            let cs = Output::new(&mut p.P0_17, Level::High, OutputDrive::Standard);
            let mut bus = SpiBusInnerFixed { spi, cs };

            if let Err(e) = audio_system_init(&mut bus, &mut hw_codec_reset_out).await {
                error!("Error initialising audio codec: {:?}", e);
                return;
            }
        }

        info!("Waiting for button press. Is playing: {}", is_playing);
        btn_1.wait().await;
        info!("Button pressed");
        is_playing = !is_playing;
        debounce_button().await;
    }
}

async fn debounce_button() {
    Timer::after(Duration::from_millis(100)).await;
}

async fn audio_system_init(
    bus: &mut impl Bus<spim::Error>,
    hw_codec_reset_out: &mut Output<'_, P0_18>,
) -> Result<(), spim::Error> {
    // drive RESET low then high
    hw_codec_reset_out.set_low();
    Timer::after(Duration::from_millis(24)).await;
    hw_codec_reset_out.set_high();

    // reset
    dsp::reset_spi(bus).await?;
    info!("System init and reset complete");

    // configure codec
    dsp::default_conf_enable_spi(bus).await?;
    info!("HW codec configured for streaming");

    Ok(())
}
