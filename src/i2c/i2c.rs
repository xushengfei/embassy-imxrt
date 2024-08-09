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
    fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        todo!();
    }

    fn write(&mut self, _address: u8, _write: &[u8]) -> Result<(), Self::Error> {
        todo!();
    }

    fn write_read(&mut self, _address: u8, _write: &[u8], _read: &mut [u8]) -> Result<(), Self::Error> {
        todo!();
    }

    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        todo!();
    }
}
