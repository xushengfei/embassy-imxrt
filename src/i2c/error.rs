//! I2C (Inter-Integrated Circuit) bus Errors

use super::i2cm::I2c;
use super::instance::Instance;

/// Error Types for I2C communication
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Timeout error.
    Timeout,
    /// Reading from i2c failed
    ReadFail,
    /// I2C Address not acked
    AddressNack,
}

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Timeout => embedded_hal_1::i2c::ErrorKind::Other,
            Self::ReadFail => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Data)
            }
            Self::AddressNack => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
            }
        }
    }
}

impl<'d, T: Instance> embedded_hal_1::i2c::ErrorType for I2c<'d, T> {
    type Error = Error;
}
