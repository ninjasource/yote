#![no_std]
#![no_main]

// A basic LED blinky program for the yote hearing aid

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    defmt::flush();

    // restart on crash
    cortex_m::peripheral::SCB::sys_reset();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Started");

    // setup peripherals for nrf5340 audio dk board
    let p = embassy_nrf::init(Default::default());

    // led - yote board
    let mut rgb1_red = Output::new(AnyPin::from(p.P0_04), Level::Low, OutputDrive::Standard);

    loop {
        rgb1_red.set_low();
        info!("Led OFF");
        Timer::after(Duration::from_millis(1000)).await;

        rgb1_red.set_high();
        info!("Led ON");
        Timer::after(Duration::from_millis(100)).await;
    }
}
