//! Implements I2C function support over flexcomm + gpios

use core::marker::PhantomData;

use embassy_hal_internal::Peripheral;
use embassy_sync::waitqueue::AtomicWaker;
use paste::paste;
use sealed::Sealed;

use crate::iopctl::IopctlPin as Pin;
use crate::{dma, interrupt};

/// I2C Master Driver
pub mod master;

/// I2C Slave Driver
pub mod slave;

/// shorthand for -> `Result<T>`
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
    /// configuration requested is not supported
    UnsupportedConfiguration,

    /// transaction failure types
    Transfer(TransferError),
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

struct Info {
    regs: &'static crate::pac::i2c0::RegisterBlock,
    index: usize,
}

trait SealedInstance {
    fn info() -> Info;
    fn index() -> usize;
}

/// shared functions between master and slave operation
#[allow(private_bounds)]
pub trait Instance: crate::flexcomm::IntoI2c + SealedInstance + Peripheral<P = Self> + 'static + Send {
    /// Interrupt for this I2C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
	$(
	    paste!{
		impl SealedInstance for crate::peripherals::[<FLEXCOMM $n>] {
		    fn info() -> Info {
			Info {
			    regs: unsafe { &*crate::pac::[<I2c $n>]::ptr() },
			    index: $n,
			}
		    }


		    #[inline]
		    fn index() -> usize {
			$n
		    }
		}

		impl Instance for crate::peripherals::[<FLEXCOMM $n>] {
		    type Interrupt = crate::interrupt::typelevel::[<FLEXCOMM $n>];
		}
	    }
	)*
    };
}

impl_instance!(0, 1, 2, 3, 4, 5, 6, 7);

const I2C_COUNT: usize = 8;
static I2C_WAKERS: [AtomicWaker; I2C_COUNT] = [const { AtomicWaker::new() }; I2C_COUNT];

/// Maximum I2C chunk size for DMA transfers
pub const MAX_I2C_CHUNK_SIZE: usize = 1024;

/// Ten bit addresses start with first byte 0b11110XXX
pub const TEN_BIT_PREFIX: u8 = 0b11110 << 3;

/// I2C interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let waker = &I2C_WAKERS[T::index()];

        let i2c = T::info().regs;

        if i2c.intstat().read().mstpending().bit_is_set() {
            i2c.intenclr().write(|w| w.mstpendingclr().set_bit());
        }

        if i2c.intstat().read().mstarbloss().bit_is_set() {
            i2c.intenclr().write(|w| w.mstarblossclr().set_bit());
        }

        if i2c.intstat().read().mstststperr().bit_is_set() {
            i2c.intenclr().write(|w| w.mstststperrclr().set_bit());
        }

        if i2c.intstat().read().slvpending().bit_is_set() {
            i2c.intenclr().write(|w| w.slvpendingclr().set_bit());
        }

        if i2c.intstat().read().slvdesel().bit_is_set() {
            i2c.intenclr().write(|w| w.slvdeselclr().set_bit());
        }

        waker.wake();
    }
}

/// io configuration trait for easier configuration
pub trait SclPin<Instance>: Pin + sealed::Sealed + Peripheral {
    /// convert the pin to appropriate function for SCL usage
    fn as_scl(&self);
}

/// io configuration trait for easier configuration
pub trait SdaPin<Instance>: Pin + sealed::Sealed + Peripheral {
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
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
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
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
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

/// I2C Master DMA trait.
#[allow(private_bounds)]
pub trait MasterDma<T: Instance>: dma::Instance {}

/// I2C Slave DMA trait.
#[allow(private_bounds)]
pub trait SlaveDma<T: Instance>: dma::Instance {}

macro_rules! impl_dma {
    ($fcn:ident, $mode:ident, $dma:ident) => {
        paste! {
            impl [<$mode Dma>]<crate::peripherals::$fcn> for crate::peripherals::$dma {}
        }
    };
}

impl_dma!(FLEXCOMM0, Slave, DMA0_CH0);
impl_dma!(FLEXCOMM0, Master, DMA0_CH1);

impl_dma!(FLEXCOMM1, Slave, DMA0_CH2);
impl_dma!(FLEXCOMM1, Master, DMA0_CH3);

impl_dma!(FLEXCOMM2, Slave, DMA0_CH4);
impl_dma!(FLEXCOMM2, Master, DMA0_CH5);

impl_dma!(FLEXCOMM3, Slave, DMA0_CH6);
impl_dma!(FLEXCOMM3, Master, DMA0_CH7);

impl_dma!(FLEXCOMM4, Slave, DMA0_CH8);
impl_dma!(FLEXCOMM4, Master, DMA0_CH9);

impl_dma!(FLEXCOMM5, Slave, DMA0_CH10);
impl_dma!(FLEXCOMM5, Master, DMA0_CH11);

impl_dma!(FLEXCOMM6, Slave, DMA0_CH12);
impl_dma!(FLEXCOMM6, Master, DMA0_CH13);

impl_dma!(FLEXCOMM7, Slave, DMA0_CH14);
impl_dma!(FLEXCOMM7, Master, DMA0_CH15);
