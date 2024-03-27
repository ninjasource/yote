#![no_std]
#![no_main]

// A basic LED blinky program for the nrf5340 Audio DK

use core::mem;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};
use nrf5340_app_pac as pac;

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

    // gpio leds
    // audio dk board
    let mut rgb1_red = Output::new(AnyPin::from(p.P0_07), Level::Low, OutputDrive::HighDrive);

    loop {
        rgb1_red.set_low();
        info!("Led OFF");
        Timer::after(Duration::from_millis(1000)).await;

        rgb1_red.set_high();
        info!("Led ON");
        Timer::after(Duration::from_millis(100)).await;
    }
}
