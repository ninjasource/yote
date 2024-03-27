#![no_std]
#![no_main]
#![allow(unused)]

// a binary used to play around with power measurements
// this is why there is so much commented out code - the entire binary is meant for experimentation

use cs47l63::driver;
use cs47l63::spec::CS47L63_DEVID;
use embassy_nrf::spis::Mode;
use embassy_nrf::Peripherals;
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

bind_interrupts!(struct Irqs {
    SERIAL3 => spim::InterruptHandler<SERIAL3>;
    I2S0 => i2s::InterruptHandler<I2S0>;
});

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    defmt::flush();

    // restart on crash
    cortex_m::peripheral::SCB::sys_reset();
}

fn setup_pins(p: Peripherals) -> Output<'static, AnyPin> {
    let _p1_00 = Input::new(p.P1_00, Pull::Up);
    let _p1_01 = Output::new(p.P1_01, Level::Low, OutputDrive::Standard);
    let _p1_02 = Input::new(p.P1_02, Pull::Down);
    let _p1_03 = Input::new(p.P1_03, Pull::Down);
    let _p1_04 = Input::new(p.P1_04, Pull::Down);
    let _p1_05 = Input::new(p.P1_05, Pull::Down);
    let _p1_06 = Input::new(p.P1_06, Pull::Down);
    let _p1_07 = Input::new(p.P1_07, Pull::Down);
    let _p1_08 = Input::new(p.P1_08, Pull::Down);
    let _p1_09 = Input::new(p.P1_09, Pull::Down);
    let _p1_10 = Input::new(p.P1_10, Pull::Down);
    let _p1_11 = Input::new(p.P1_11, Pull::Down);
    let _p1_12 = Input::new(p.P1_12, Pull::Down);
    let _p1_13 = Input::new(p.P1_13, Pull::Down);
    let _p1_14 = Input::new(p.P1_14, Pull::Down);
    let _p1_15 = Input::new(p.P1_15, Pull::Down);

    let _p0_00 = Input::new(p.P0_00, Pull::Down);
    let _p0_01 = Input::new(p.P0_01, Pull::Down);
    let _p0_02 = Input::new(p.P0_02, Pull::Down);
    let _p0_03 = Input::new(p.P0_03, Pull::Down);
    let p0_04 = Output::new(AnyPin::from(p.P0_04), Level::Low, OutputDrive::Standard); // red led
    let _p0_05 = Input::new(p.P0_05, Pull::Down);
    let _p0_06 = Input::new(p.P0_06, Pull::Down);
    let _p0_07 = Input::new(p.P0_07, Pull::Down);
    let _p0_08 = Output::new(p.P0_08, Level::Low, OutputDrive::Standard);
    let _p0_09 = Output::new(p.P0_09, Level::Low, OutputDrive::Standard);
    let _p0_10 = Input::new(p.P0_10, Pull::Down);
    let _p0_11 = Input::new(p.P0_11, Pull::Down);
    let _p0_12 = Output::new(p.P0_12, Level::Low, OutputDrive::Standard);
    let _p0_13 = Input::new(p.P0_13, Pull::Down);
    let _p0_14 = Output::new(p.P0_14, Level::Low, OutputDrive::Standard);
    let _p0_15 = Input::new(p.P0_15, Pull::Down);
    let _p0_16 = Output::new(p.P0_16, Level::Low, OutputDrive::Standard);
    let _p0_17 = Output::new(p.P0_17, Level::Low, OutputDrive::Standard);
    let _p0_18 = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    let _p0_19 = Input::new(p.P0_19, Pull::Down);
    let _p0_20 = Output::new(p.P0_20, Level::Low, OutputDrive::Standard);
    let _p0_21 = Input::new(p.P0_21, Pull::Down);
    let _p0_22 = Input::new(p.P0_22, Pull::Down);
    let _p0_23 = Input::new(p.P0_23, Pull::Down);
    let _p0_24 = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
    let _p0_25 = Input::new(p.P0_25, Pull::Down);
    let _p0_26 = Input::new(p.P0_26, Pull::Down);
    let _p0_27 = Input::new(p.P0_27, Pull::Down);
    let _p0_28 = Input::new(p.P0_28, Pull::Down);
    let _p0_29 = Input::new(p.P0_29, Pull::Down);
    let _p0_30 = Input::new(p.P0_30, Pull::Down);
    let _p0_31 = Input::new(p.P0_31, Pull::Down);
    p0_04
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Started");

    /*
    // change app core clock from 64mhz to 128mhz for improved performance
    let clock: pac::CLOCK_S = unsafe { mem::transmute(()) };
    clock.hfclkctrl.write(|w| w.hclk().div1());
    //info!("Set app core to 128mhz");


        // enable flash cache for improved performance
        let cache: pac::CACHE_S = unsafe { mem::transmute(()) };
        cache.enable.write(|w| w.enable().enabled());
        //info!("Enabled flash cache");
    */

    // setup peripherals for nrf5340 audio dk board
    let mut p = embassy_nrf::init(Default::default());

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

    // gpio leds
    // audio dk board
    // let mut rgb1_red = Output::new(AnyPin::from(p.P0_07), Level::Low, OutputDrive::HighDrive);

    // yote board
    let mut rgb1_red = Output::new(AnyPin::from(p.P0_04), Level::Low, OutputDrive::Standard);

    // gpio setup
    let mut hw_codec_reset_out =
        Output::new(AnyPin::from(p.P0_18), Level::High, OutputDrive::Standard);

    // drive RESET low then high
    hw_codec_reset_out.set_low();
    Timer::after(Duration::from_millis(24)).await;
    hw_codec_reset_out.set_high();

    //let mut rgb1_red = setup_pins(p);

    loop {
        rgb1_red.set_low();

        //info!("Led OFF");
        Timer::after(Duration::from_millis(1000)).await;

        hw_codec_reset_out.set_low();
        // rgb1_red.set_high();
        //info!("Led ON");

        /*
                // spi setup
                let mut config = spim::Config::default();
                config.frequency = Frequency::M8;
                let mut spi = spim::Spim::new(
                    &mut p.SERIAL3,
                    Irqs,
                    &mut p.P0_08,
                    &mut p.P0_10,
                    &mut p.P0_09,
                    config,
                );
                let mut cs_codec = Output::new(&mut p.P0_17, Level::High, OutputDrive::Standard);

                let reg_with_write_bit = CS47L63_DEVID | 0x80000000;
                cs_codec.set_low();
                spi.write(&reg_with_write_bit.to_be_bytes()).await;
                cs_codec.set_high();
        */
        Timer::after(Duration::from_millis(200)).await;

        // drop(spi);
        //  drop(cs_codec);
    }
}
