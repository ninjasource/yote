use core::ops::DerefMut;
use cs47l63::hw_interface::Bus;
use embassy_nrf::peripherals::P0_17;
use embassy_nrf::{
    gpio::{AnyPin, Output},
    peripherals::SERIAL3,
    spim::{self, Instance, Spim},
};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    mutex::{Mutex, MutexGuard},
};
use embassy_time::{Duration, Timer};

const ZEROS: [u8; 4] = 0_u32.to_be_bytes();

pub struct SharedBus {
    inner: Mutex<NoopRawMutex, SpiBusInner>,
}

struct SpiBusInner {
    pub spi: Spim<'static, SERIAL3>,
    pub cs: Output<'static, AnyPin>,
}

pub struct SpiBusInnerFixed<'a> {
    pub spi: Spim<'a, SERIAL3>,
    pub cs: Output<'a, P0_17>, // hardcoded to p0_17 so that it can be dropped easily to save power
}

pub struct BusImpl<'a> {
    bus: MutexGuard<'a, NoopRawMutex, SpiBusInner>,
}

impl SharedBus {
    pub fn new(spi: Spim<'static, SERIAL3>, cs: Output<'static, AnyPin>) -> Self {
        Self {
            inner: Mutex::new(SpiBusInner { spi, cs }),
        }
    }

    pub async fn borrow(&self) -> BusImpl<'_> {
        let bus = self.inner.lock().await;
        BusImpl { bus }
    }
}

impl<'a> Bus<spim::Error> for SpiBusInnerFixed<'a> {
    async fn read(&mut self, reg: u32) -> Result<u32, spim::Error> {
        self.cs.set_low();
        let result = read_inner(&mut self.spi, reg).await;
        self.cs.set_high();
        result
    }

    async fn write(&mut self, reg: u32, val: u32) -> Result<(), spim::Error> {
        self.cs.set_low();
        let result = write_inner(&mut self.spi, reg, val).await;
        self.cs.set_high();
        result
    }

    async fn write_block(&mut self, reg: u32, val: &[u8]) -> Result<(), spim::Error> {
        self.cs.set_low();
        let result = write_block_inner(&mut self.spi, reg, val).await;
        self.cs.set_high();
        result
    }

    async fn delay_ms(&self, millis: u64) {
        Timer::after(Duration::from_millis(millis)).await;
    }
}

// implementation of the hardware interface to be used by the dsp driver
impl<'a> Bus<spim::Error> for BusImpl<'a> {
    async fn read(&mut self, reg: u32) -> Result<u32, spim::Error> {
        let spi_bus = self.bus.deref_mut();
        spi_bus.cs.set_low();
        let result = read_inner(&mut spi_bus.spi, reg).await;
        spi_bus.cs.set_high();
        result
    }

    async fn write(&mut self, reg: u32, val: u32) -> Result<(), spim::Error> {
        let spi_bus = self.bus.deref_mut();
        spi_bus.cs.set_low();
        let result = write_inner(&mut spi_bus.spi, reg, val).await;
        spi_bus.cs.set_high();
        result
    }

    async fn write_block(&mut self, reg: u32, bytes: &[u8]) -> Result<(), spim::Error> {
        let spi_bus = self.bus.deref_mut();
        spi_bus.cs.set_low();
        let result = write_block_inner(&mut spi_bus.spi, reg, bytes).await;
        spi_bus.cs.set_high();
        result
    }

    async fn delay_ms(&self, millis: u64) {
        Timer::after(Duration::from_millis(millis)).await;
    }
}

async fn read_inner<T: Instance>(spi: &mut Spim<'_, T>, reg: u32) -> Result<u32, spim::Error> {
    // set the write bit
    let reg_with_write_bit = reg | 0x80000000;
    let write = reg_with_write_bit.to_be_bytes();
    let mut read: [u8; 4] = [0; 4];

    spi.write(&write).await?; // write register
    spi.write(&ZEROS).await?; // padding
    spi.read(&mut read).await?; // read register result

    Ok(u32::from_be_bytes(read))
}

async fn write_block_inner<T: Instance>(
    spi: &mut Spim<'_, T>,
    reg: u32,
    bytes: &[u8],
) -> Result<(), spim::Error> {
    spi.write(&reg.to_be_bytes()).await?; // write register
    spi.write(&ZEROS).await?; // padding
    spi.write(bytes).await?; // write bytes
    Ok(())
}

async fn write_inner<T: Instance>(
    spi: &mut Spim<'_, T>,
    reg: u32,
    val: u32,
) -> Result<(), spim::Error> {
    spi.write(&reg.to_be_bytes()).await?; // write register
    spi.write(&ZEROS).await?; // padding
    spi.write(&val.to_be_bytes()).await?; // write value
    Ok(())
}
