//! I2C (Inter-Integrated Circuit) bus HAL object

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral};

use super::config::Config;
use super::instance::Instance;

/// I2C Struct
#[allow(private_bounds)]
pub struct I2c<'d, T: Instance> {
    _flexcomm: PhantomData<&'d mut T>,
    config: Config,
}

#[allow(private_bounds)]
impl<'d, T: Instance> I2c<'d, T> {
    /// Create a new I2C controller instance from one fo the Flexcomm ports
    pub fn new(_instance: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        into_ref!(_instance);

        let i2c = Self {
            _flexcomm: PhantomData,
            config,
        };

        T::init();

        i2c
    }

    fn i2c_master_set() {
        let i2cregs = T::i2c_regs();

        // Set I2C clock to ~2MHz, 16MHz SFRO / 8 = 2MHz => 400 kHz bus?
        i2cregs.clkdiv().write(|w| unsafe { w.divval().bits(160) });
        i2cregs
            .msttime()
            .write(|w| unsafe { w.mstsclhigh().bits(0).mstscllow().bits(0) });
        i2cregs.timeout().write(|w| unsafe { w.to().bits(4096 >> 4) });
        i2cregs.intenset().write(|w| unsafe { w.bits(0) });

        i2cregs.cfg().write(|w| w.msten().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}
    }
}

impl<'d, T: Instance> embedded_hal_1::i2c::I2c for I2c<'d, T> {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        Self::i2c_master_set();

        // Procedure from 24.3.1.2 pg 546
        let i2cregs = T::i2c_regs();

        i2cregs
            .mstdat()
            .write(|w| unsafe { w.data().bits(address << 1 | 0x01) });
        i2cregs.mstctl().write(|w| w.mststart().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        for index in 0..read.len() {
            while i2cregs.stat().read().mstpending().bit_is_clear() {}
            read[index] = (i2cregs.mstdat().read().bits() & 0xFF) as u8;
        }

        i2cregs.mstctl().write(|w| w.mststop().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        Ok(())
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        Self::i2c_master_set();

        // Procedure from 24.3.1.1 pg 545
        let i2cregs = T::i2c_regs();

        i2cregs.mstdat().write(|w| unsafe { w.data().bits(address << 1 | 0) });
        i2cregs.mstctl().write(|w| w.mststart().set_bit());

        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        for byte in write.iter() {
            i2cregs.mstdat().write(|w| unsafe { w.data().bits(*byte) });
            i2cregs.mstctl().write(|w| w.mstcontinue().set_bit());
            while i2cregs.stat().read().mstpending().bit_is_clear() {}
        }

        i2cregs.mstctl().write(|w| w.mststop().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        Ok(())
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        Self::i2c_master_set();

        let i2cregs = T::i2c_regs();

        i2cregs.mstdat().write(|w| unsafe { w.data().bits(address << 1 | 0) });
        i2cregs.mstctl().write(|w| w.mststart().set_bit());

        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        for byte in write.iter() {
            i2cregs.mstdat().write(|w| unsafe { w.data().bits(*byte) });
            i2cregs.mstctl().write(|w| w.mstcontinue().set_bit());
            while i2cregs.stat().read().mstpending().bit_is_clear() {}
        }

        i2cregs
            .mstdat()
            .write(|w| unsafe { w.data().bits(address << 1 | 0x01) });
        i2cregs.mstctl().write(|w| w.mststart().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        for index in 0..read.len() {
            while i2cregs.stat().read().mstpending().bit_is_clear() {}
            read[index] = i2cregs.mstdat().read().data().bits();
        }

        i2cregs.mstctl().write(|w| w.mststop().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        Ok(())
    }

    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        todo!();
    }
}
