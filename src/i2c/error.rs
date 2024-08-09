use super::{i2c::I2c, instance::Instance};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Timeout error.
    Timeout,
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
