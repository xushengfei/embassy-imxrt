//! Universal Asynchronous Receiver Transmitter (UART) driver.
//!

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::flexcomm::Mode;
use crate::gpio::{AnyPin, GpioPin as Pin};
use crate::iopctl::{DriveMode, DriveStrength, Inverter, IopctlPin, Pull, SlewRate};
use crate::pac::usart0::cfg::{Clkpol, Datalen, Loop, Paritysel as Parity, Stoplen, Syncen, Syncmst};
use crate::pac::usart0::ctl::Cc;

type Baudrate = u32;

/// Summary
///
/// This code implements very basic functionality of the UART.- blocking reading/ writing a single buffer of data
/// TODO: Default register mapping is non-secure. Yet to find the mapping for secure address "0x50106000" in embassy 658 pac
/// TODO: Add flow control
///
mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

impl<T: Pin> sealed::Sealed for T {}

trait SealedInstance {
    fn regs() -> &'static crate::pac::usart0::RegisterBlock;
}

/// Uart
#[allow(private_bounds)]
pub trait Instance: crate::flexcomm::UsartPeripheral + SealedInstance + Peripheral<P = Self> + 'static + Send {}

macro_rules! impl_instance {
    ($fc:ident, $usart:ident) => {
        impl SealedInstance for crate::peripherals::$fc {
            fn regs() -> &'static crate::pac::usart0::RegisterBlock {
                unsafe { &*crate::pac::$usart::ptr() }
            }
        }

        impl Instance for crate::peripherals::$fc {}
    };
}

impl_instance!(FLEXCOMM0, Usart0);
impl_instance!(FLEXCOMM1, Usart1);
impl_instance!(FLEXCOMM2, Usart2);
impl_instance!(FLEXCOMM3, Usart3);
impl_instance!(FLEXCOMM4, Usart4);
impl_instance!(FLEXCOMM5, Usart5);
impl_instance!(FLEXCOMM6, Usart6);
impl_instance!(FLEXCOMM7, Usart7);

/// io configuration trait for Uart Tx configuration
pub trait TxPin<T: Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for Uart Tx  usage
    fn as_tx(&self);
}

/// io configuration trait for Uart Rx configuration
pub trait RxPin<T: Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for Uart Rx  usage
    fn as_rx(&self);
}

/// Uart struct to hold the uart configuration
pub struct Uart<'a, T: Instance> {
    _inner: PeripheralRef<'a, T>,
    _tx: Option<PeripheralRef<'a, AnyPin>>,
    _rx: Option<PeripheralRef<'a, AnyPin>>,
}

/// UART general config
#[derive(Clone, Copy)]
pub struct GeneralConfig {
    /// Baudrate of the Uart
    pub baudrate: Baudrate,
    /// data length
    pub data_bits: Datalen,
    /// Parity
    pub parity: Parity,
    /// Stop bits
    pub stop_bits: Stoplen,
}

impl Default for GeneralConfig {
    /// Default configuration for single channel sampling.
    fn default() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: Datalen::Bit8,
            parity: Parity::NoParity,
            stop_bits: Stoplen::Bit1,
        }
    }
}

/// UART `MCU_specific` config
#[derive(Clone, Copy)]
pub struct UartMcuSpecificConfig {
    /// Polarity of the clock
    pub clock_polarity: Clkpol,
    /// Sync/ Async operation selection
    pub operation: Syncen,
    /// Sync master/slave mode selection (only applicable in sync mode)
    pub sync_mode_master_select: Syncmst,
    /// USART continuous Clock generation enable in synchronous master mode.
    pub continuous_clock: Cc,
    /// Normal/ loopback mode
    pub loopback_mode: Loop,
}

impl Default for UartMcuSpecificConfig {
    /// Default configuration for single channel sampling.
    fn default() -> Self {
        Self {
            clock_polarity: Clkpol::FallingEdge,
            operation: Syncen::AsynchronousMode,
            sync_mode_master_select: Syncmst::Slave,
            continuous_clock: Cc::ClockOnCharacter,
            loopback_mode: Loop::Normal,
        }
    }
}

/// Specific information regarding transfer errors
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransferError {
    /// Read error
    UsartRxError,
    /// Buffer overflow
    UsartRxRingBufferOverrun,
    /// Noise error in Rx
    UsartNoiseError,
    /// Framing error in Rx
    UsartFramingError,
    /// Parity error in Rx
    UsartParityError,
}

/// Uart Errors
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// propagating a lower level flexcomm error
    Flex(crate::flexcomm::Error),

    /// Failure
    Fail,
    /// Invalid argument
    InvalidArgument,

    /// Uart baud rate cannot be supported with the given clock
    UsartBaudrateNotSupported,

    /// Transaction failure errors
    Transfer(TransferError),
}
/// shorthand for -> Result<T>
pub type Result<T> = core::result::Result<T, Error>;

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

impl<'a, T: Instance> Uart<'a, T> {
    /// Bidirectional uart
    pub fn new(
        _inner: impl Peripheral<P = T> + 'a,
        tx: impl Peripheral<P = impl TxPin<T>> + 'a,
        rx: impl Peripheral<P = impl RxPin<T>> + 'a,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;
        T::enable(clock);
        T::set_mode(Mode::Usart)?;

        into_ref!(_inner);
        into_ref!(tx);
        into_ref!(rx);

        tx.as_tx();
        rx.as_rx();

        let this = Self {
            _inner,
            _tx: Some(tx.map_into()),
            _rx: Some(rx.map_into()),
        };

        this.set_uart_tx_fifo();
        this.set_uart_rx_fifo();
        this.set_uart_baudrate(&general_config)?;
        this.set_uart_config(&general_config, &mcu_spec_config);

        Ok(this)
    }

    /// Unidirectional Uart - Tx only
    pub fn new_tx_only(
        _inner: impl Peripheral<P = T> + 'a,
        tx: impl Peripheral<P = impl TxPin<T>> + 'a,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;
        T::enable(clock);
        T::set_mode(Mode::Usart)?;

        into_ref!(_inner);
        into_ref!(tx);
        tx.as_tx();

        let this = Self {
            _inner,
            _tx: Some(tx.map_into()),
            _rx: None,
        };

        this.set_uart_tx_fifo();
        this.set_uart_baudrate(&general_config)?;
        this.set_uart_config(&general_config, &mcu_spec_config);

        Ok(this)
    }

    /// Unidirectional Uart - Rx only
    pub fn new_rx_only(
        _inner: impl Peripheral<P = T> + 'a,
        rx: impl Peripheral<P = impl RxPin<T>> + 'a,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;
        T::enable(clock);
        T::set_mode(Mode::Usart)?;

        into_ref!(_inner);
        into_ref!(rx);
        rx.as_rx();

        let this = Self {
            _inner,
            _tx: None,
            _rx: Some(rx.map_into()),
        };

        this.set_uart_rx_fifo();
        this.set_uart_baudrate(&general_config)?;
        this.set_uart_config(&general_config, &mcu_spec_config);

        Ok(this)
    }

    fn get_fc_freq(&self) -> u32 {
        // Todo: Make it generic for any clock
        // Since the FC clock is hardcoded to Sfro, this freq is returned.
        // sfro : 16MHz, // ffro: 48MHz
        16_000_000
    }

    fn set_uart_baudrate(&self, gen_config: &GeneralConfig) -> Result<()> {
        let baudrate_bps = gen_config.baudrate;
        let source_clock_hz = self.get_fc_freq(); // TODO: replace this with the call to flexcomm_getClkFreq()

        if baudrate_bps == 0 || source_clock_hz == 0 {
            return Err(Error::InvalidArgument);
        }

        // If synchronous master mode is enabled, only configure the BRG value.
        if T::regs().cfg().read().syncen().is_synchronous_mode() {
            // Master
            if T::regs().cfg().read().syncmst().is_master() {
                // Calculate the BRG value
                let brgval = (source_clock_hz / baudrate_bps) - 1;

                // SAFETY: unsafe only used for .bits()
                T::regs().brg().write(|w| unsafe { w.brgval().bits(brgval as u16) });
            }
        } else {
            // Smaller values of OSR can make the sampling position within a data bit less accurate and may
            // potentially cause more noise errors or incorrect data.
            let (_, osr, brg) = (8..16).rev().fold(
                (u32::MAX, u32::MAX, u32::MAX),
                |(best_diff, best_osr, best_brg), osrval| {
                    let brgval = (source_clock_hz / ((osrval + 1) * baudrate_bps)) - 1;
                    let diff;

                    if brgval > 65535 {
                        (best_diff, best_osr, best_brg)
                    } else {
                        // Calculate the baud rate based on the BRG value
                        let baudrate = source_clock_hz / ((osrval + 1) * (brgval + 1));

                        // Calculate the difference between the
                        // current baud rate and the desired baud rate
                        diff = (baudrate as i32 - baudrate_bps as i32).unsigned_abs();

                        // Check if the current calculated difference is the best so far
                        if diff < best_diff {
                            (diff, osrval, brgval)
                        } else {
                            (best_diff, best_osr, best_brg)
                        }
                    }
                },
            );

            // Value over range
            if brg > 65535 {
                return Err(Error::UsartBaudrateNotSupported);
            }

            // SAFETY: unsafe only used for .bits()
            T::regs().osr().write(|w| unsafe { w.osrval().bits(osr as u8) });

            // SAFETY: unsafe only used for .bits()
            T::regs().brg().write(|w| unsafe { w.brgval().bits(brg as u16) });
        }

        Ok(())
    }

    fn set_uart_tx_fifo(&self) {
        T::regs()
            .fifocfg()
            .modify(|_, w| w.emptytx().set_bit().enabletx().enabled());

        // clear FIFO error
        T::regs().fifostat().write(|w| w.txerr().set_bit());
    }

    fn set_uart_rx_fifo(&self) {
        T::regs()
            .fifocfg()
            .modify(|_, w| w.emptyrx().set_bit().enablerx().enabled());

        // clear FIFO error
        T::regs().fifostat().write(|w| w.rxerr().set_bit());
    }

    fn set_uart_config(&self, gen_config: &GeneralConfig, uart_mcu_spec_config: &UartMcuSpecificConfig) {
        T::regs().cfg().write(|w| w.enable().disabled());

        T::regs().cfg().modify(|_, w| {
            w.datalen()
                .variant(gen_config.data_bits)
                .stoplen()
                .variant(gen_config.stop_bits)
                .paritysel()
                .variant(gen_config.parity)
                .loop_()
                .variant(uart_mcu_spec_config.loopback_mode)
                .syncen()
                .variant(uart_mcu_spec_config.operation)
                .clkpol()
                .variant(uart_mcu_spec_config.clock_polarity)
        });

        T::regs().cfg().modify(|_, w| w.enable().enabled());
    }

    /// Deinitializes a USART instance.
    pub fn deinit(&self) -> Result<()> {
        // This function waits for TX complete, disables TX and RX, and disables the USART clock
        while T::regs().stat().read().txidle().bit_is_clear() {
            // When 0, indicates that the transmitter is currently in the process of sending data.
        }

        // Disable interrupts
        T::regs().fifointenclr().modify(|_, w| {
            w.txerr()
                .set_bit()
                .rxerr()
                .set_bit()
                .txlvl()
                .set_bit()
                .rxlvl()
                .set_bit()
        });

        // Disable dma requests
        T::regs()
            .fifocfg()
            .modify(|_, w| w.dmatx().clear_bit().dmarx().clear_bit());

        // Disable peripheral
        T::regs().cfg().modify(|_, w| w.enable().disabled());

        Ok(())
    }

    /// Read RX data register using a blocking method.
    /// This function polls the RX register, waits for the RX register to be full or for RX FIFO to
    /// have data and read data from the TX register.
    /// Note for testing purpose : Blocking read API, that can receive a max of data of 8 bytes.
    /// The actual data expected to be received should be sent as "len"
    pub fn read_blocking(&self, buf: &mut [u8]) -> Result<()> {
        // Check if rxFifo is not enabled
        if T::regs().fifocfg().read().enablerx().is_disabled() {
            return Err(Error::Fail);
        } else {
            // rxfifo is enabled
            for b in buf.iter_mut() {
                // loop until rxFifo has some data to read
                while T::regs().fifostat().read().rxnotempty().bit_is_clear() {}

                // Now that there is some data in the rxFifo, read it
                // Let's verify the rxFifo status flags
                if T::regs().fifostat().read().rxerr().bit_is_set() {
                    T::regs().fifocfg().modify(|_, w| w.emptyrx().set_bit());
                    T::regs().fifostat().modify(|_, w| w.rxerr().set_bit());
                    return Err(Error::Transfer(TransferError::UsartRxError));
                }

                let mut read_status = false; // false implies failure
                let mut generic_status = Error::Fail;

                // clear all status flags
                if T::regs().stat().read().parityerrint().bit_is_set() {
                    T::regs().stat().modify(|_, w| w.parityerrint().clear_bit_by_one());
                    generic_status = Error::Transfer(TransferError::UsartParityError);
                } else if T::regs().stat().read().framerrint().bit_is_set() {
                    T::regs().stat().modify(|_, w| w.framerrint().clear_bit_by_one());
                    generic_status = Error::Transfer(TransferError::UsartFramingError);
                } else if T::regs().stat().read().rxnoiseint().bit_is_set() {
                    T::regs().stat().modify(|_, w| w.rxnoiseint().clear_bit_by_one());
                    generic_status = Error::Transfer(TransferError::UsartNoiseError);
                } else {
                    // No error, proceed with read
                    read_status = true;
                }

                if read_status {
                    // read the data from the rxFifo
                    *b = T::regs().fiford().read().rxdata().bits() as u8;
                } else {
                    return Err(generic_status);
                }
            }
        }

        Ok(())
    }

    /// Writes to the TX register using a blocking method.
    /// This function polls the TX register, waits for the TX register to be empty or for the TX FIFO
    /// to have room and writes data to the TX buffer.
    /// Note for testing purpose : Blocking write API, that can send a max of data of 8 bytes.
    /// The actual data expected to be sent should be sent as "len"
    pub fn write_blocking(&self, buf: &[u8]) -> Result<()> {
        // Check whether txFIFO is enabled
        if T::regs().fifocfg().read().enabletx().is_disabled() {
            return Err(Error::Fail);
        } else {
            for x in buf {
                // Loop until txFIFO get some space for new data
                while T::regs().fifostat().read().txnotfull().bit_is_clear() {}
                // SAFETY: unsafe only used for .bits()
                T::regs().fifowr().write(|w| unsafe { w.txdata().bits(u16::from(*x)) });
            }
            // Wait to finish transfer
            while T::regs().stat().read().txidle().bit_is_clear() {}
        }

        Ok(())
    }
}

macro_rules! impl_uart_tx {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl TxPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_tx(&self) {
                // UM11147 table 507 pg 495
                self.set_function(crate::iopctl::Function::$fn)
                    .set_pull(Pull::None)
                    .enable_input_buffer()
                    .set_slew_rate(SlewRate::Standard)
                    .set_drive_strength(DriveStrength::Normal)
                    .disable_analog_multiplex()
                    .set_drive_mode(DriveMode::PushPull)
                    .set_input_inverter(Inverter::Disabled);
            }
        }
    };
}

macro_rules! impl_uart_rx {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl RxPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_rx(&self) {
                // UM11147 table 507 pg 495
                self.set_function(crate::iopctl::Function::$fn)
                    .set_pull(Pull::None)
                    .enable_input_buffer()
                    .set_slew_rate(SlewRate::Standard)
                    .set_drive_strength(DriveStrength::Normal)
                    .disable_analog_multiplex()
                    .set_drive_mode(DriveMode::PushPull)
                    .set_input_inverter(Inverter::Disabled);
            }
        }
    };
}
// Flexcomm0 Uart TX/Rx
impl_uart_tx!(PIO0_1, F1, FLEXCOMM0); //Tx
impl_uart_rx!(PIO0_2, F1, FLEXCOMM0); //Rx
impl_uart_tx!(PIO3_1, F5, FLEXCOMM0); //Tx
impl_uart_rx!(PIO3_2, F5, FLEXCOMM0); //Rx

// Flexcomm1 Uart TX/Rx
impl_uart_tx!(PIO0_8, F1, FLEXCOMM1); //Tx
impl_uart_rx!(PIO0_9, F1, FLEXCOMM1); //Rx
impl_uart_tx!(PIO7_26, F1, FLEXCOMM1); //Tx
impl_uart_rx!(PIO7_27, F1, FLEXCOMM1); //Rx

// Flexcomm2 Uart Tx/Rx
impl_uart_tx!(PIO0_15, F1, FLEXCOMM2); //Tx
impl_uart_rx!(PIO0_16, F1, FLEXCOMM2); //Rx
impl_uart_tx!(PIO7_30, F5, FLEXCOMM2); //Tx
impl_uart_rx!(PIO7_31, F5, FLEXCOMM2); //Rx

// Flexcomm3 Uart Tx/Rx
impl_uart_tx!(PIO0_22, F1, FLEXCOMM3); //Tx
impl_uart_rx!(PIO0_23, F1, FLEXCOMM3); //Rx

// Flexcomm4 Uart Tx/Rx
impl_uart_tx!(PIO0_29, F1, FLEXCOMM4); //Tx
impl_uart_rx!(PIO0_30, F1, FLEXCOMM4); //Rx

// Flexcomm5 Uart Tx/Rx
impl_uart_tx!(PIO1_4, F1, FLEXCOMM5); //Tx
impl_uart_rx!(PIO1_5, F1, FLEXCOMM5); //Rx
impl_uart_tx!(PIO3_16, F5, FLEXCOMM5); //Tx
impl_uart_rx!(PIO3_17, F5, FLEXCOMM5); //Rx

// Flexcomm6 Uart Tx/Rx
impl_uart_tx!(PIO3_26, F1, FLEXCOMM6); //Tx
impl_uart_rx!(PIO3_27, F1, FLEXCOMM6); //Rx

// Flexcomm7 Uart Tx/Rx
impl_uart_tx!(PIO4_1, F1, FLEXCOMM7); //Tx
impl_uart_rx!(PIO4_2, F1, FLEXCOMM7); //Rx
