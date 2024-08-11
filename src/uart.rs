//!UART
//!
#![macro_use]

use crate::pac::usart0;
use core::marker::PhantomData;
use mimxrt685s_pac as pac;

use embassy_hal_internal::{into_ref, PeripheralRef};

use pac::usart0::RegisterBlock;
//pub use pac::usart0::{config::Parity, Baudrate::Baudrate};
//pub use pac::usart0::{config::Parity, Baudrate::Baudrate};
pub use pac::usart0::cfg::Paritysel as Parity;

/// UART Data Error
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum DataError {
    /// Parity error
    Parity,
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// Overrun error
    Overrun,
}

/// Uart Config Error
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ConfigError {
    /// Invalid Uart Config
    InvalidConfig,
}
