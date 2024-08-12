//!Universal Asynchronous Receiver Transmitter (UART) driver.
//!

#![macro_use]

use core::marker::PhantomData;
use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use mimxrt685s_pac::dma0::errint0::Err;

use mimxrt685s_pac as pac;

use crate::pac::usart0;
use pac::usart0::RegisterBlock;
use pac::Interrupt;

// Re-export SVD variants to allow user to directly set values.
pub use pac::usart0::cfg::Datalen;
pub use pac::usart0::cfg::Paritysel as Parity;
pub use pac::usart0::cfg::Stoplen;
pub use u32 as Baudrate;

//use crate::interrupt::typelevel::Interrupt;
use crate::interrupt;

//TODO: Gpio crate should implement "GpioPin, AnyPin". Temporary definition of "GpioPin" is taken as u8
use u8 as GpioPin;
use u8 as AnyPin;

/// Uart Config
pub struct Config {
    pub baudrate: Baudrate,
    pub data_bits: Datalen,
    pub parity: Parity,
    pub stop_bits: Stoplen,
}

/// Uart Config Error
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ConfigError {
    /// Invalid Uart Config
    InvalidConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            baudrate: 300000000,
            data_bits: Datalen::Bit8,
            parity: Parity::NoParity,
            stop_bits: Stoplen::Bit1,
        }
    }
}

/// Error source flags
bitflags::bitflags! {
    pub struct ErrorSource:u32{
        const FRAMING = 1 << 13;
        const PARITY = 1 << 14;
        const NOISE = 1 << 15;
        const AUTOBAUD = 1 << 16;
    }
}

impl ErrorSource {
    #[inline]
    fn check(self) -> Result<(), Error> {
        if self.contains(ErrorSource::FRAMING) {
            Err(Error::Framing)
        } else if self.contains(ErrorSource::PARITY) {
            Err(Error::Parity)
        } else if self.contains(ErrorSource::NOISE) {
            Err(Error::Noise)
        } else if self.contains(ErrorSource::AUTOBAUD) {
            Err(Error::Autobaud)
        } else {
            Ok(())
        }
    }
}

/// UART error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing Error
    Framing,
    /// Parity Error
    Parity,
    /// Noise Error
    Noise,
    /// Autobaud condition
    Autobaud,
}

/*
// TODO: "Instance" is not defined in rt6, so find another way
/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // TODO: Add Interrupt handling
    }
}
*/

/// USART peripheral instance trait.
#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

/// UART driver.
pub struct Uart<'d> {
    tx: UartTx<'d>,
    rx: UartRx<'d>,
}

/// Transmitter half of the UART driver.
pub struct UartTx<'d> {
    //info
    //state
    // sclk: PeripheralRef<'d, AnyPin>,
    tx: PeripheralRef<'d, AnyPin>,
    cts: Option<PeripheralRef<'d, AnyPin>>,
    _p: PhantomData<&'d ()>,
}

/// Receiver half of the UART driver.
pub struct UartRx<'d> {
    //info
    //state
    //sclk: PeripheralRef<'d, AnyPin>,
    rx: PeripheralRef<'d, AnyPin>,
    rts: Option<PeripheralRef<'d, AnyPin>>,
    _p: PhantomData<&'d ()>,
}

impl<'d> SetConfig for Uart<'d> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.tx.set_config(config)?;
        self.rx.set_config(config)
    }
    // Ok(())
}

impl<'d> SetConfig for UartTx<'d> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, _config: &Self::Config) -> Result<(), Self::ConfigError> {
        // Add code
        Ok(())
    }
}

impl<'d> SetConfig for UartRx<'d> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, _config: &Self::Config) -> Result<(), Self::ConfigError> {
        // Add code
        Ok(())
    }
}

impl<'d> Uart<'d> {
    /// Create a new UART driver.
    fn new(
        _peri: PeripheralRef<'d, AnyPin>,
        _sclk: PeripheralRef<'d, AnyPin>,
        _tx: PeripheralRef<'d, AnyPin>,
        _rx: PeripheralRef<'d, AnyPin>,
        _cts: Option<PeripheralRef<'d, AnyPin>>,
        _rts: Option<PeripheralRef<'d, AnyPin>>,
        _config: Config,
    ) -> Result<Self, ConfigError> {
        // Your initialization code here

        let mut this = Self {
            tx: UartTx::new(_tx, _cts).unwrap(),
            rx: UartRx::new(_rx, _rts).unwrap(),
        };
        // Return an instance of Uart
        Ok(this)
    }

    /// Split the Uart into a transmitter and receiver, which is
    /// particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UartTx<'d>, UartRx<'d>) {
        (self.tx, self.rx)
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }
}

impl<'d> UartTx<'d> {
    fn new(_tx: PeripheralRef<'d, AnyPin>, _cts: Option<PeripheralRef<'d, AnyPin>>) -> Result<Self, ConfigError> {
        // Your initialization code here

        let mut this = Self {
            tx: _tx,
            cts: _cts,
            _p: PhantomData,
        };
        // Return an instance of UartTx
        Ok(this)
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        // Your code here
        Ok(())
    }
}

impl<'d> UartRx<'d> {
    fn new(_rx: PeripheralRef<'d, AnyPin>, _rts: Option<PeripheralRef<'d, AnyPin>>) -> Result<Self, ConfigError> {
        // Your initialization code here

        let mut this = Self {
            rx: _rx,
            rts: _rts,
            _p: PhantomData,
        };
        // Return an instance of UartTx
        Ok(this)
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // Your code here
        Ok(())
    }
}
