#![allow(unused_imports)]

use super::config::{
    ASP1_ENABLE, CLOCK_CONFIGURATION, COMPRESSION_ENABLE_CONFIGURE, EQUALIZER_ENABLE_CONFIGURE,
    FLL_DISABLE, FLL_ENABLE, GPIO_CONFIGURATION, OUTPUT_ENABLE_BASIC, OUTPUT_ENABLE_COMPRESSION,
    OUTPUT_ENABLE_EQ, OUTPUT_ENABLE_PASSTHOUGH, PDM_MIC_ENABLE_CONFIGURE,
    PDM_MIC_ENABLE_CONFIGURE_PASSTHOUGH, SOFT_RESET,
};
use super::shared_bus::{BusImpl, SharedBus};
use cs47l63::{driver, hw_interface::Bus, registers::output_signal_path::volume_ctrl};
use embassy_nrf::spim;
use embassy_time::{Duration, Timer};

/// resets and initialises the device
/// this assumes that the user has driven the reset pin low, then waited 4ms then high again
pub async fn reset(shared_bus: &SharedBus) -> Result<(), spim::Error> {
    // hard reset with patch
    let mut bus = shared_bus.borrow().await;
    driver::reset(&mut bus).await?;
    drop(bus);

    // soft reset
    reg_conf_write(shared_bus, &SOFT_RESET).await?;
    Timer::after(Duration::from_micros(3000)).await;
    Ok(())
}

/// resets and initialises the device
/// this assumes that the user has driven the reset pin low, then waited 4ms then high again
pub async fn reset_spi(bus: &mut impl Bus<spim::Error>) -> Result<(), spim::Error> {
    // hard reset with patch
    driver::reset(bus).await?;

    // soft reset
    reg_conf_write_spi(bus, &SOFT_RESET).await?;
    Timer::after(Duration::from_micros(3000)).await;
    Ok(())
}

pub async fn default_conf_enable(shared_bus: &SharedBus) -> Result<(), spim::Error> {
    reg_conf_write(shared_bus, &CLOCK_CONFIGURATION).await?;
    reg_conf_write(shared_bus, &GPIO_CONFIGURATION).await?;
    reg_conf_write(shared_bus, &OUTPUT_ENABLE_PASSTHOUGH).await?;
    reg_conf_write(shared_bus, &PDM_MIC_ENABLE_CONFIGURE_PASSTHOUGH).await?;
    reg_conf_write(shared_bus, &EQUALIZER_ENABLE_CONFIGURE).await?;
    reg_conf_write(shared_bus, &COMPRESSION_ENABLE_CONFIGURE).await?;

    // fll toggle
    reg_conf_write(shared_bus, &FLL_DISABLE).await?;
    Timer::after(Duration::from_micros(1000)).await;
    reg_conf_write(shared_bus, &FLL_ENABLE).await?;
    Ok(())
}

pub async fn default_conf_enable_spi(bus: &mut impl Bus<spim::Error>) -> Result<(), spim::Error> {
    reg_conf_write_spi(bus, &CLOCK_CONFIGURATION).await?;
    reg_conf_write_spi(bus, &GPIO_CONFIGURATION).await?;
    reg_conf_write_spi(bus, &OUTPUT_ENABLE_PASSTHOUGH).await?;
    reg_conf_write_spi(bus, &PDM_MIC_ENABLE_CONFIGURE_PASSTHOUGH).await?;
    reg_conf_write_spi(bus, &EQUALIZER_ENABLE_CONFIGURE).await?;
    reg_conf_write_spi(bus, &COMPRESSION_ENABLE_CONFIGURE).await?;

    // fll toggle
    reg_conf_write_spi(bus, &FLL_DISABLE).await?;
    Timer::after(Duration::from_micros(1000)).await;
    reg_conf_write_spi(bus, &FLL_ENABLE).await?;
    Ok(())
}

pub async fn volume_mute<E>(bus: &mut impl Bus<E>, mute: bool) -> Result<(), E> {
    let mut out_vol: volume_ctrl::Out1LVolume1 =
        bus.read(volume_ctrl::Out1LVolume1::REG).await?.into();
    out_vol.mute = mute;
    out_vol.update = true;
    let [reg, val] = out_vol.serialize();
    bus.write(reg, val).await?;
    Ok(())
}

pub async fn volume_adjust<E>(bus: &mut impl Bus<E>, adjustment_db: i32) -> Result<i32, E> {
    const MAX_VOLUME_DB: i32 = 64;
    const MAX_VOLUME_REG_VAL: i32 = 0x80;

    let mut out_vol: volume_ctrl::Out1LVolume1 =
        bus.read(volume_ctrl::Out1LVolume1::REG).await?.into();

    // The adjustment is in dB, 1 bit equals 0.5 dB,
    // so multiply by 2 to get increments of 1 dB
    let volume = (out_vol.volume as i32) + (adjustment_db * 2);

    if volume < 0 {
        return Ok(-64);
    } else if volume > MAX_VOLUME_REG_VAL {
        return Ok(0);
    }

    out_vol.volume = volume as u8;
    out_vol.update = true;
    let [reg, val] = out_vol.serialize();
    bus.write(reg, val).await?;
    Ok(volume / 2 - MAX_VOLUME_DB)
}

async fn reg_conf_write_spi(
    bus: &mut impl Bus<spim::Error>,
    config: &[[u32; 2]],
) -> Result<(), spim::Error> {
    for [reg, value] in config {
        bus.write(*reg, *value).await?;
    }

    Ok(())
}

async fn reg_conf_write(shared_bus: &SharedBus, config: &[[u32; 2]]) -> Result<(), spim::Error> {
    let mut bus = shared_bus.borrow().await;

    for [reg, value] in config {
        bus.write(*reg, *value).await?;
    }

    Ok(())
}
