//! Implements I2C function support over flexcomm + gpios
use core::marker::PhantomData;

use embassy_sync::waitqueue::AtomicWaker;
use sealed::Sealed;

use crate::flexcomm::{self, FlexcommLowLevel};
use crate::iopctl::IopctlPin as Pin;
use crate::Peripheral;

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

/// Flexcomm configured for I2C usage
#[allow(private_bounds)]
pub struct I2cBus<'p> {
    info: I2CBusInfo,
    _lifetime: PhantomData<&'p ()>,
}

/// Struct to keep track of all relevant I2C bus info
/// this allows removal of `<FC: Instance>` from the typestate
#[derive(Copy, Clone)]
pub struct I2CBusInfo {
    /// Pointer to Flexcomm Registers
    // All flexcomms point to same register block
    pub regs: &'static flexcomm::FlexcommRegisters,
    /// Pointer to I2C registers
    // All I2Cs point to same register block
    pub i2cregs: &'static flexcomm::I2cRegisters,
    /// Pointer to instance (`FCn`) specific waker
    pub waker: &'static AtomicWaker,
}

#[allow(private_bounds)]
impl<'p> I2cBus<'p> {
    // keep impl peripheral so that we can take mutable references (like when creating GPIOs)
    // check link from felipe
    /// use Flexcomm fc as a blocking I2c Bus
    pub fn new_blocking<F: Instance>(_fc: impl Peripheral<P = F> + 'p, clk: flexcomm::Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(flexcomm::Mode::I2c)?;
        Ok(Self {
            info: F::i2c_bus_info(),
            _lifetime: PhantomData::<&'p ()>,
        })
    }

    /// use Flexcomm fc as an async I2c Bus
    pub fn new_async<F: Instance>(_fc: impl Peripheral<P = F> + 'p, clk: flexcomm::Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(flexcomm::Mode::I2c)?;
        // SAFETY: flexcomm interrupt should be managed through this
        //         interface only
        unsafe { F::enable_interrupt() };
        Ok(Self {
            info: F::i2c_bus_info(),
            _lifetime: PhantomData::<&'p ()>,
        })
    }

    /// retrieve active bus registers
    pub fn i2c(&self) -> &'static flexcomm::I2cRegisters {
        self.info.i2cregs
    }

    /// return a waker
    pub fn waker(&self) -> &'static AtomicWaker {
        self.info.waker
    }

    /// return bus info
    pub fn info(&self) -> I2CBusInfo {
        self.info
    }
}

/// Trait to seal away a FC instance specific I2CBus
pub(crate) trait SealedInstance: FlexcommLowLevel {
    /// returns the Instance specific info for the given I2C bus
    fn i2c_bus_info() -> I2CBusInfo;
}

/// shared functions between master and slave operation
#[allow(private_bounds)]
pub trait Instance: SealedInstance {}

macro_rules! impl_instance {
    ($fc:ident, $i2c:ident) => {
        impl SealedInstance for crate::peripherals::$fc {
            fn i2c_bus_info() -> I2CBusInfo {
                I2CBusInfo {
                    regs: crate::peripherals::$fc::reg(),
                    i2cregs: crate::peripherals::$fc::i2c(),
                    waker: crate::peripherals::$fc::waker(),
                }
            }
        }
        impl Instance for crate::peripherals::$fc {}
    };
}

impl_instance!(FLEXCOMM0, I2c0);
impl_instance!(FLEXCOMM1, I2c1);
impl_instance!(FLEXCOMM2, I2c2);
impl_instance!(FLEXCOMM3, I2c3);
impl_instance!(FLEXCOMM4, I2c4);
impl_instance!(FLEXCOMM5, I2c5);
impl_instance!(FLEXCOMM6, I2c6);
impl_instance!(FLEXCOMM7, I2c7);

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
