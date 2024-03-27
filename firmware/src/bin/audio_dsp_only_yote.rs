#![no_std]
#![no_main]

use cs47l63::hw_interface::Bus;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Pin, Pull};
use embassy_nrf::peripherals::P0_18;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals::SERIAL3,
    spim::{self, Frequency},
};
use embassy_time::{Duration, Timer};
use yote::hw_dsp::dsp;
use yote::hw_dsp::shared_bus::SpiBusInnerFixed;

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
    let mut p = embassy_nrf::init(Default::default());

    let mut hw_codec_reset_out = Output::new(p.P0_18, Level::High, OutputDrive::Standard);
    let mut _pmic_iset_out = Output::new(AnyPin::from(p.P1_01), Level::Low, OutputDrive::Standard);

    // gpio buttons
    let btn = Input::new(p.P1_00.degrade(), Pull::Up);

    // NOTE: THIS DOES NOT SEEM TO WORK
    // let btn_1 = InputChannel::new(p.GPIOTE_CH0.degrade(), btn, InputChannelPolarity::HiToLo);

    let mut led = Output::new(AnyPin::from(p.P0_04), Level::Low, OutputDrive::HighDrive);
    info!("Wait for things to settle");

    // wait for things to settle
    Timer::after(Duration::from_millis(50)).await;

    led.set_high();
    Timer::after(Duration::from_millis(20)).await;
    led.set_low();
    Timer::after(Duration::from_millis(100)).await;

    info!("Ready");

    let mut is_playing = false;

    info!("Resetting audio codec");
    hw_codec_reset_out.set_low();
    Timer::after(Duration::from_millis(24)).await;
    hw_codec_reset_out.set_high();

    loop {
        info!("Waiting for button press. Is playing: {}", is_playing);

        let mut reset_counter = 0;

        'outer: loop {
            Timer::after(Duration::from_millis(100)).await;

            if btn.is_low() {
                info!("Button pressed");

                info!("Resetting audio codec");
                hw_codec_reset_out.set_low();
                Timer::after(Duration::from_millis(24)).await;
                hw_codec_reset_out.set_high();

                is_playing = !is_playing;
                loop {
                    // debounce
                    Timer::after(Duration::from_millis(100)).await;

                    reset_counter += 1;
                    if btn.is_high() {
                        // wait for user to release the button

                        // holding down the button for more than a second will reset the device
                        if reset_counter > 10 {
                            cortex_m::peripheral::SCB::sys_reset();
                        }

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

                            if let Err(e) =
                                audio_system_init(&mut bus, &mut hw_codec_reset_out).await
                            {
                                error!("Error initialising audio codec: {:?}", e);
                                return;
                            }
                        }

                        flash_led(is_playing, &mut led).await;
                        break 'outer;
                    }
                }
            }
        }
    }
}

async fn flash_led(is_playing: bool, led: &mut Output<'_, AnyPin>) {
    for _ in 0..5 {
        led.set_high();
        Timer::after(Duration::from_millis(20)).await;
        led.set_low();
        Timer::after(Duration::from_millis(100)).await;
    }

    if !is_playing {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led.set_low();
    }
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
