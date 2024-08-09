use super::{config::Config, instance::Instance};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embedded_hal_1::i2c::Operation;

/// I2C Struct
pub struct I2c<'d, T: Instance> {
    _flexcomm: PhantomData<&'d mut T>,
    config: Config,
}

impl<'d, T: Instance> I2c<'d, T> {
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
        todo!();
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        todo!();
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        todo!();
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        todo!();
    }
}
