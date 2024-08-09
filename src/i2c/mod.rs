//! I2C (Inter-Integrated Circuit) bus support

use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embedded_hal_1::i2c::Operation;

#[allow(non_camel_case_types)]
pub enum Frequency {
    F100_kHz,
    F400_kHz,
}

pub struct Config {
    /// Frequency for I2C Communications
    pub frequency: Frequency,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::F100_kHz,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Timeout error.
    Timeout,
}

trait SealedInstance {
    /// Returns a reference to peripheral's register block.
    fn regs() -> &'static crate::pac::flexcomm2::RegisterBlock;

    /// Initializes power and clocks to peripheral.
    fn init() -> ();
}

/// WWDT instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance {}

/// I2C Struct
pub struct I2c<'d, T: Instance> {
    _flexcomm: PhantomData<&'d mut T>,
    config: Config,
}

// Cortex-M33 Flexcomm2
impl SealedInstance for crate::peripherals::FLEXCOMM2 {
    fn regs() -> &'static crate::pac::flexcomm2::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm2::ptr() }
    }

    fn init() {}
}
impl Instance for crate::peripherals::FLEXCOMM2 {}

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

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Timeout => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, T: Instance> embedded_hal_1::i2c::ErrorType for I2c<'d, T> {
    type Error = Error;
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
