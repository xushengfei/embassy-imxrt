//! Implements I2C function support over flexcomm + gpios

use sealed::Sealed;

use crate::iopctl::IopctlPin as Pin;

/// I2C Master Driver
pub mod master;

/// I2C Slave Driver
pub mod slave;

/// shorthand for -> Result<T>
pub type Result<T> = core::result::Result<T, Error>;

/// Bus speed (nominal SCL, no clock stretching)
pub enum Speed {
    /// 100 kbit/s
    Standard,

    /// 400 kbit/s
    Fast,

    /// 1 Mbit/s
    FastPlus,

    /// 3.4Mbit/s only available for slave devices
    High,
}

/// specific information regarding transfer errors
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransferError {
    /// Timeout error
    Timeout,
    /// Reading from i2c failed
    ReadFail,
    /// Writing to i2c failed
    WriteFail,
    /// I2C Address not ACK'd
    AddressNack,
    /// Bus level arbitration loss
    ArbitrationLoss,
    /// Address + Start/Stop error
    StartStopError,
    /// state mismatch or other internal register unexpected state
    OtherBusError,
}

/// Error information type
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// propagating a lower level flexcomm error
    Flex(crate::flexcomm::Error),

    /// configuration requested is not supported
    UnsupportedConfiguration,

    /// transaction failure types
    Transfer(TransferError),
}

// implementing from allows ? operator from flexcomm::Result<T>
impl From<crate::flexcomm::Error> for Error {
    fn from(value: crate::flexcomm::Error) -> Self {
        Error::Flex(value)
    }
}

impl From<TransferError> for Error {
    fn from(value: TransferError) -> Self {
        Error::Transfer(value)
    }
}

mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

impl<T: Pin> sealed::Sealed for T {}

/// shared functions between master and slave operation
#[allow(private_bounds)]
pub trait Instance: crate::flexcomm::I2cPeripheral {}
impl Instance for crate::peripherals::FLEXCOMM0 {}
impl Instance for crate::peripherals::FLEXCOMM1 {}
impl Instance for crate::peripherals::FLEXCOMM2 {}
impl Instance for crate::peripherals::FLEXCOMM3 {}
impl Instance for crate::peripherals::FLEXCOMM4 {}
impl Instance for crate::peripherals::FLEXCOMM5 {}
impl Instance for crate::peripherals::FLEXCOMM6 {}
impl Instance for crate::peripherals::FLEXCOMM7 {}

/// io configuration trait for easier configuration
pub trait SclPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for SCL usage
    fn as_scl(&self);
}

/// io configuration trait for easier configuration
pub trait SdaPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for SDA usage
    fn as_sda(&self);
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: Sealed {}

/// Blocking mode.
pub struct Blocking;
impl Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl Sealed for Async {}
impl Mode for Async {}

// flexcomm <-> Pin function map
macro_rules! impl_scl {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl SclPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_scl(&self) {
                // UM11147 table 556 pg 550
                self.set_function(crate::iopctl::Function::$fn)
                    .set_pull(crate::iopctl::Pull::None)
                    .enable_input_buffer()
                    .set_slew_rate(crate::gpio::SlewRate::Slow)
                    .set_drive_strength(crate::gpio::DriveStrength::Full)
                    .disable_analog_multiplex()
                    .set_drive_mode(crate::gpio::DriveMode::OpenDrain)
                    .set_input_inverter(crate::gpio::Inverter::Disabled);
            }
        }
    };
}
macro_rules! impl_sda {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl SdaPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_sda(&self) {
                // UM11147 table 556 pg 550
                self.set_function(crate::iopctl::Function::$fn)
                    .set_pull(crate::iopctl::Pull::None)
                    .enable_input_buffer()
                    .set_slew_rate(crate::gpio::SlewRate::Slow)
                    .set_drive_strength(crate::gpio::DriveStrength::Full)
                    .disable_analog_multiplex()
                    .set_drive_mode(crate::gpio::DriveMode::OpenDrain)
                    .set_input_inverter(crate::gpio::Inverter::Disabled);
            }
        }
    };
}

// Flexcomm0 GPIOs -
impl_scl!(PIO0_1, F1, FLEXCOMM0);
impl_sda!(PIO0_2, F1, FLEXCOMM0);

impl_scl!(PIO3_1, F5, FLEXCOMM0);
impl_sda!(PIO3_2, F5, FLEXCOMM0);
impl_sda!(PIO3_3, F5, FLEXCOMM0);
impl_scl!(PIO3_4, F5, FLEXCOMM0);

// Flexcomm1 GPIOs -
impl_scl!(PIO0_8, F1, FLEXCOMM1);
impl_sda!(PIO0_9, F1, FLEXCOMM1);
impl_sda!(PIO0_10, F1, FLEXCOMM1);
impl_scl!(PIO0_11, F1, FLEXCOMM1);

impl_scl!(PIO7_26, F1, FLEXCOMM1);
impl_sda!(PIO7_27, F1, FLEXCOMM1);
impl_sda!(PIO7_28, F1, FLEXCOMM1);
impl_scl!(PIO7_29, F1, FLEXCOMM1);

// Flexcomm2 GPIOs -
impl_scl!(PIO0_15, F1, FLEXCOMM2);
impl_sda!(PIO0_16, F1, FLEXCOMM2);
impl_sda!(PIO0_17, F1, FLEXCOMM2);
impl_scl!(PIO0_18, F1, FLEXCOMM2);

impl_sda!(PIO4_8, F5, FLEXCOMM2);

impl_scl!(PIO7_30, F5, FLEXCOMM2);
impl_sda!(PIO7_31, F5, FLEXCOMM2);

// Flexcomm3 GPIOs -
impl_scl!(PIO0_22, F1, FLEXCOMM3);
impl_sda!(PIO0_23, F1, FLEXCOMM3);
impl_sda!(PIO0_24, F1, FLEXCOMM3);
impl_scl!(PIO0_25, F1, FLEXCOMM3);

// Flexcomm4 GPIOs -
impl_scl!(PIO0_29, F1, FLEXCOMM4);
impl_sda!(PIO0_30, F1, FLEXCOMM4);
impl_sda!(PIO0_31, F1, FLEXCOMM4);
impl_scl!(PIO1_0, F1, FLEXCOMM4);

// Flexcomm5 GPIOs -
impl_scl!(PIO1_4, F1, FLEXCOMM5);
impl_sda!(PIO1_5, F1, FLEXCOMM5);
impl_sda!(PIO1_6, F1, FLEXCOMM5);
impl_scl!(PIO1_7, F1, FLEXCOMM5);

impl_scl!(PIO3_16, F4, FLEXCOMM5);
impl_sda!(PIO3_17, F4, FLEXCOMM5);
impl_sda!(PIO3_18, F4, FLEXCOMM5);
impl_scl!(PIO3_22, F5, FLEXCOMM5);

// Flexcomm6 GPIOs -
impl_scl!(PIO3_26, F1, FLEXCOMM6);
impl_sda!(PIO3_27, F1, FLEXCOMM6);
impl_sda!(PIO3_28, F1, FLEXCOMM6);
impl_scl!(PIO3_29, F1, FLEXCOMM6);

// Flexcomm7 GPIOs -
impl_scl!(PIO4_1, F1, FLEXCOMM7);
impl_sda!(PIO4_2, F1, FLEXCOMM7);
impl_sda!(PIO4_3, F1, FLEXCOMM7);
impl_scl!(PIO4_4, F1, FLEXCOMM7);
