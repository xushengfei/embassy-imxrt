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
}

impl<'d, T: Instance> embedded_hal_1::i2c::I2c for I2c<'d, T> {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        // Procedure from 24.3.1.2 pg 546
        let i2cregs = T::i2c_regs();
        i2cregs.cfg().write(|w| w.msten().set_bit());

        i2cregs.mstdat().write(|w| unsafe { w.data().bits(address | 0x80) });
        i2cregs.mstctl().write(|w| w.mststart().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        for index in 0..read.len() {
            while i2cregs.stat().read().mstpending().bit_is_clear() {}
            read[index] = (i2cregs.mstdat().read().bits() & 0xFF) as u8;
        }

        i2cregs.mstctl().write(|w| w.mststop().set_bit());

        Ok(())
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        // Procedure from 24.3.1.1 pg 545
        let i2cregs = T::i2c_regs();
        i2cregs.cfg().write(|w| w.msten().set_bit());

        i2cregs.mstdat().write(|w| unsafe { w.data().bits(address & 0x7F) });
        i2cregs.mstctl().write(|w| w.mststart().set_bit());
        while i2cregs.stat().read().mstpending().bit_is_clear() {}

        for byte in write.iter() {
            i2cregs.mstdat().write(|w| unsafe { w.data().bits(*byte) });
            while i2cregs.stat().read().mstpending().bit_is_clear() {}
        }

        i2cregs.mstctl().write(|w| w.mststop().set_bit());

        Ok(())
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.write(address, write)?;
        self.read(address, read)?;
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
