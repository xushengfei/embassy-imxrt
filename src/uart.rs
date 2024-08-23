//!Universal Asynchronous Receiver Transmitter (UART) driver.
//!

#![macro_use]

use core::marker::PhantomData;

use crate::pac::usart0;
use mimxrt685s_pac as pac;
use pac::usart0::RegisterBlock;

// Re-export SVD variants to allow user to directly set values.
pub use pac::usart0::cfg::Datalen;
pub use pac::usart0::cfg::Paritysel as Parity;
pub use pac::usart0::cfg::Stoplen;
pub use u32 as Baudrate;

///Assumptions
/// - This is a basic test code to verify a very basic functionality of the UART.- reading/ writing a single buffer of data
/// Flexcomm 0 will be hard coded for now. Plus clock.
/// Using the flexcomm 0 base address for uart. i.e using Usart0 only. This is by default mapped to 0x40106000 (Non-secure).
/// TODO: Yet to find the mapping for secure address "0x50106000" in embassy 658 pac
/// IOCTL for the uart pins will be hard coded.
/// Add the hardcoded part will be added in uart_setting.rs file for easy separation. This is a temp file which will be refactored out once flexcomm, clocks, gpios are fully implemented
/// Adding more customizable, generic code will be the next step
/// Also features like DMA, async data transfer, etc will be added later.
///

/// UartRx struct to hold the uart configuration
pub struct UartRx {
    pub baudrate: Baudrate,
    pub data_bits: Datalen,
    pub parity: Parity,
    pub stop_bits: Stoplen,
    pub flexcomm_freq: u32,
}

/// Generic status enum to return the status of the uart read/write operations
/// Todo: In the vendor file fsl_common.h, there is an enum defined enum _status_groups{},
/// that can be used to define the status of all the peripherals in a standard way.
/// Since that is missing in the pac, I am defining a temp status
#[derive(PartialEq)]
pub enum GenericStatus {
    // Generic status
    Success,
    Fail,
    ReadOnly,
    OutOfRange,
    InvalidArgument,
    Timeout,
    NoTransferInProgress,
    //uart specific peripheral status
    USART_TxBusy = 5700,
    USART_RxBusy,
    USART_TxIdle,
    USART_RxIdle,
    USART_TxError,
    USART_RxError,
    USART_RxRingBufferOverrun,
    USART_NoiseError,
    USART_FramingError,
    USART_ParityError,
    USART_BaudrateNotSupport,
}

impl UartRx {
    pub fn new() -> Self {
        UartRx {
            baudrate: 115200,
            data_bits: Datalen::Bit8,
            parity: Parity::NoParity,
            stop_bits: Stoplen::Bit1,
            flexcomm_freq: 100_000_000,
        }
    }
    /// Exposing a method to access reg internally with the assumption that only the uart0 is being used
    fn reg(&self) -> &'static pac::usart0::RegisterBlock {
        unsafe { &*(pac::Usart0::ptr() as *const pac::usart0::RegisterBlock) }
    }

    /// Use this API to prog all the registers for the uart0 - assuming flexcomm0, clocks, ioctl are already set
    pub fn open(&self) {
        self.set_uart_config();
        self.set_uart_rx_fifo();
    }

    pub fn close(&self) {}

    /// Blocking read API, that can receive a max of data of 8 bytes. The actual data expected to be received should be sent as "len"
    pub fn read_blocking(&self, buf: &mut [u8; 8], len: u32) -> GenericStatus {
        if len > 8 {
            return GenericStatus::InvalidArgument;
        }

        // Check if rxFifo is not enabled
        if self.reg().fifocfg().read().enablerx().bit_is_clear() {
            return GenericStatus::Fail;
        } else {
            // rxfifo is enabled
            for i in 0..len {
                // loop until rxFifo has some data to read
                while self.reg().fifostat().read().rxnotempty().bit_is_clear() {}

                // Now that there is some data in the rxFifo, read it
                // Let's verify the rxFifo status flags
                if self.reg().fifostat().read().rxerr().bit_is_set() {
                    self.reg().fifocfg().write(|w| w.emptyrx().set_bit());
                    self.reg().fifostat().write(|w| w.rxerr().set_bit());
                    return GenericStatus::USART_RxError;
                }

                // Save the receive status flag to check later.
                let rx_status = self.reg().stat().read().bits();
                let mut generic_status = GenericStatus::Success;

                // clear all status flags

                //TODO: Note that bits 13,14 and 15 (FrameErrInt, ParityErrInt, ExNoiseErrInt) of uart::Stat reg is R/W1C, but in the  imxrt632s-pac, the read for these bits is not implemented..
                // Need to add the implementation for these bits in the pac file

                if rx_status & (1 << 14) != 0 {
                    //writing to it will clear the status since it is W1C
                    self.reg().stat().write(|w| w.parityerrint().set_bit());
                    generic_status = GenericStatus::USART_ParityError;
                }
                if rx_status & (1 << 13) != 0 {
                    //writing to it will clear the status since it is W1C
                    self.reg().stat().write(|w| w.framerrint().set_bit());
                    generic_status = GenericStatus::USART_FramingError;
                }
                if rx_status & (1 << 15) != 0 {
                    //writing to it will clear the status since it is W1C
                    self.reg().stat().write(|w| w.rxnoiseint().set_bit());
                    generic_status = GenericStatus::USART_NoiseError;
                }

                if generic_status == GenericStatus::Success {
                    // read the data from the rxFifo
                    //todo: check if this conversion is correct
                    buf[i as usize] = self.reg().fiford().read().rxdata().bits() as u8;
                } else {
                    return generic_status;
                }
            }
        }

        return GenericStatus::Success;
    }

    fn set_uart_config(&self) {
        // setting the uart data len
        if self.data_bits == Datalen::Bit8 {
            self.reg().cfg().write(|w| w.datalen().bit_8());
        } else if self.data_bits == Datalen::Bit7 {
            self.reg().cfg().write(|w| w.datalen().bit_7());
        } else if self.data_bits == Datalen::Bit9 {
            self.reg().cfg().write(|w| w.datalen().bit_9());
        }

        //setting the uart stop bits
        if self.stop_bits == Stoplen::Bit1 {
            self.reg().cfg().write(|w| w.stoplen().bit_1());
        } else if self.stop_bits == Stoplen::Bits2 {
            self.reg().cfg().write(|w| w.stoplen().bits_2());
        }

        //setting the uart parity
        if self.parity == Parity::NoParity {
            self.reg().cfg().write(|w| w.paritysel().no_parity());
        } else if self.parity == Parity::EvenParity {
            self.reg().cfg().write(|w| w.paritysel().even_parity());
        } else if self.parity == Parity::OddParity {
            self.reg().cfg().write(|w| w.paritysel().odd_parity());
        }
    }

    fn set_uart_rx_fifo(&self) {
        // setting the rx fifo
        self.reg().fifocfg().write(|w| w.emptyrx().set_bit());
        self.reg().fifocfg().write(|w| w.enablerx().enabled());
    }
}
