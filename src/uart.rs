//!Universal Asynchronous Receiver Transmitter (UART) driver.
//!

#![macro_use]

use crate::iopctl::*;
use mimxrt685s_pac as pac;

use crate::iopctl::IopctlPin as Pin;
use crate::PeripheralRef;

// Re-export SVD variants to allow user to directly set values.
pub use pac::usart0::cfg::Datalen;
pub use pac::usart0::cfg::Paritysel as Parity;
pub use pac::usart0::cfg::Stoplen;
pub use u32 as Baudrate;

pub use pac::usart0::cfg::Clkpol;
pub use pac::usart0::cfg::Loop;
/// Syncen : Sync/ Async mode selection
pub use pac::usart0::cfg::Syncen;
/// Syncmst : Sync master/slave mode selection (only applicable in sync mode)
pub use pac::usart0::cfg::Syncmst;
pub use pac::usart0::ctl::Cc;

/// Todo: Will be used when the uart is fully implemented - both tx and rx. Right now only Rx is implemented
pub use pac::usart0::fifocfg::Enablerx;
pub use pac::usart0::fifocfg::Enabletx;

/// Assumptions
/// This code implements very basic functionality of the UART.- blocking reading/ writing a single buffer of data
/// TODO: Default register mapping is non-secure. Yet to find the mapping for secure address "0x50106000" in embassy 658 pac
/// TODO: Add flow control
///

/// Pin function number.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Function {
    /// Function 0
    F0,
    /// Function 1
    F1,
    /// Function 2
    F2,
    /// Function 3
    F3,
    /// Function 4
    F4,
    /// Function 5
    F5,
    /// Function 6
    F6,
    /// Function 7
    F7,
    /// Function 8
    F8,
}

mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

impl<T: Pin> sealed::Sealed for T {}

/// Uart
#[allow(private_bounds)]
pub trait UartAny<const FC: usize>: crate::flexcomm::UsartPeripheral {}
impl UartAny<0> for crate::peripherals::FLEXCOMM0 {}
impl UartAny<1> for crate::peripherals::FLEXCOMM1 {}
impl UartAny<2> for crate::peripherals::FLEXCOMM2 {}
impl UartAny<3> for crate::peripherals::FLEXCOMM3 {}
impl UartAny<4> for crate::peripherals::FLEXCOMM4 {}
impl UartAny<5> for crate::peripherals::FLEXCOMM5 {}
impl UartAny<6> for crate::peripherals::FLEXCOMM6 {}
impl UartAny<7> for crate::peripherals::FLEXCOMM7 {}

/// io configuration trait for configuration
pub trait UartPin<const FC: usize>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for Uart Tx/Rx  usage
    fn as_txrx(&self);
}

/// Uart struct to hold the uart configuration
pub struct Uart<'a, const FC: usize, T: UartAny<FC>, Tx: UartPin<FC>, Rx: UartPin<FC>> {
    bus: crate::flexcomm::UsartBus<'a, T>,
    _tx: PeripheralRef<'a, Tx>,
    _rx: PeripheralRef<'a, Rx>,
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
            baudrate: 115200,
            data_bits: Datalen::Bit8,
            parity: Parity::NoParity,
            stop_bits: Stoplen::Bit1,
        }
    }
}

/// UART MCU_specific config
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

/// Generic status enum to return the status of the uart read/write operations
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GenericStatus {
    /// propagating a lower level flexcomm error
    Flex(crate::flexcomm::Error),

    /// Generic status
    /// Success
    Success,
    /// Failure
    Fail,
    /// Invalid argument
    InvalidArgument,

    /// Uart baud rate cannot be supported with the given clock
    UsartBaudrateNotSupported,
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
/// shorthand for -> Result<T>
pub type Result<T> = core::result::Result<T, GenericStatus>;

// implementing from allows ? operator from flexcomm::Result<T>
impl From<crate::flexcomm::Error> for GenericStatus {
    fn from(value: crate::flexcomm::Error) -> Self {
        GenericStatus::Flex(value)
    }
}

impl<'a, const FC: usize, T: UartAny<FC, P = T>, Tx: UartPin<FC, P = Tx>, Rx: UartPin<FC, P = Rx>>
    Uart<'a, FC, T, Tx, Rx>
{
    /// use flexcomm fc with Pins scl, sda as an I2C Master bus, configuring to speed and pull
    pub fn new(
        fc: T,
        tx: Tx,
        rx: Rx,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;

        let bus = crate::flexcomm::UsartBus::new(fc, clock)?;

        tx.as_txrx();
        rx.as_txrx();

        let this = Self {
            bus,
            _tx: tx.into_ref(),
            _rx: rx.into_ref(),
        };

        let result = this.set_uart_baudrate(&general_config);
        if result != GenericStatus::Success {
            return Err(result);
        }
        this.set_uart_tx_fifo();
        this.set_uart_rx_fifo();
        this.set_uart_config(&general_config, &mcu_spec_config);

        Ok(this)
    }

    fn get_fc_freq(&self) -> u32 {
        // Todo: Make it generic for any clock
        // Since the FC clock is hardcoded to Sfro, this freq is returned.
        //sfro : 0xf42400, //ffro: 0x2dc6c00
        0xf42400
    }

    fn set_uart_baudrate(&self, gen_config: &GeneralConfig) -> GenericStatus {
        let bus = &self.bus;
        let baudrate_bps = gen_config.baudrate;
        let source_clock_hz = self.get_fc_freq(); // TODO: replace this with the call to flexcomm_getClkFreq()

        let mut best_diff: u32 = 0xFFFFFFFF;
        let mut best_osrval: u32 = 0xF;
        let mut best_brgval: u32 = 0xFFFFFFFF;
        let mut diff: u32;

        if baudrate_bps == 0 || source_clock_hz == 0 {
            return GenericStatus::InvalidArgument;
        }

        //If synchronous master mode is enabled, only configure the BRG value.
        if bus.usart().cfg().read().syncen().is_synchronous_mode() {
            // Master
            if bus.usart().cfg().read().syncmst().is_master() {
                // Calculate the BRG value
                let mut brgval = source_clock_hz / baudrate_bps;
                brgval -= 1u32;
                unsafe { bus.usart().brg().write(|w| w.brgval().bits(brgval as u16)) };
            }
        } else {
            //Smaller values of OSR can make the sampling position within a data bit less accurate and may
            //potentially cause more noise errors or incorrect data.
            for osrval in (8..=0xF).rev() {
                let brgval = (source_clock_hz / ((osrval + 1u32) * baudrate_bps)) - 1u32;
                if brgval > 0xFFFFu32 {
                    continue;
                }
                // Calculate the baud rate based on the BRG value
                let baudrate = source_clock_hz / ((osrval + 1u32) * (brgval + 1u32));

                // Calculate the difference between the current baud rate and the desired baud rate
                if baudrate > baudrate_bps {
                    diff = baudrate - baudrate_bps;
                } else {
                    diff = baudrate_bps - baudrate;
                }

                // Check if the current calculated difference is the best so far
                if diff < best_diff {
                    best_diff = diff;
                    best_osrval = osrval;
                    best_brgval = brgval;
                }
            }

            // Value over range
            if best_brgval > 0xFFFFu32 {
                return GenericStatus::UsartBaudrateNotSupported;
            }

            unsafe {
                bus.usart().osr().write(|w| w.osrval().bits(best_osrval as u8));
                bus.usart().brg().write(|w| w.brgval().bits(best_brgval as u16));
            }
        }

        GenericStatus::Success
    }

    fn set_uart_tx_fifo(&self) {
        let bus = &self.bus;
        bus.usart()
            .fifocfg()
            .modify(|_, w| w.emptytx().set_bit().enabletx().enabled());

        if bus.usart().fifocfg().read().enabletx().bit_is_clear() {
            info!("Error: TX FIFO is not enabled");
        }

        // clear FIFO error
        bus.usart().fifostat().write(|w| w.txerr().set_bit());
    }

    fn set_uart_rx_fifo(&self) {
        let bus = &self.bus;
        bus.usart()
            .fifocfg()
            .modify(|_, w| w.emptyrx().set_bit().enablerx().enabled());

        if bus.usart().fifocfg().read().enablerx().bit_is_clear() {
            info!("Error: RX FIFO is not enabled");
        }

        // clear FIFO error
        bus.usart().fifostat().write(|w| w.rxerr().set_bit());
    }

    fn set_uart_config(&self, gen_config: &GeneralConfig, uart_mcu_spec_config: &UartMcuSpecificConfig) {
        let bus = &self.bus;
        bus.usart().cfg().write(|w| w.enable().disabled());

        // setting the uart data len
        if gen_config.data_bits == Datalen::Bit8 {
            bus.usart().cfg().modify(|_, w| w.datalen().bit_8());
        } else if gen_config.data_bits == Datalen::Bit7 {
            bus.usart().cfg().modify(|_, w| w.datalen().bit_7());
        } else if gen_config.data_bits == Datalen::Bit9 {
            bus.usart().cfg().modify(|_, w| w.datalen().bit_9());
        }

        //setting the uart stop bits
        if gen_config.stop_bits == Stoplen::Bit1 {
            bus.usart().cfg().modify(|_, w| w.stoplen().bit_1());
        } else if gen_config.stop_bits == Stoplen::Bits2 {
            bus.usart().cfg().modify(|_, w| w.stoplen().bits_2());
        }

        //setting the uart parity
        if gen_config.parity == Parity::NoParity {
            bus.usart().cfg().modify(|_, w| w.paritysel().no_parity());
        } else if gen_config.parity == Parity::EvenParity {
            bus.usart().cfg().modify(|_, w| w.paritysel().even_parity());
        } else if gen_config.parity == Parity::OddParity {
            bus.usart().cfg().modify(|_, w| w.paritysel().odd_parity());
        }

        // setting mcu specific uart config
        if uart_mcu_spec_config.loopback_mode == Loop::Normal {
            bus.usart().cfg().modify(|_, w| w.loop_().normal());
        } else if uart_mcu_spec_config.loopback_mode == Loop::Loopback {
            bus.usart().cfg().modify(|_, w| w.loop_().loopback());
        }

        if uart_mcu_spec_config.operation == Syncen::AsynchronousMode {
            bus.usart().cfg().modify(|_, w| w.syncen().asynchronous_mode());
        } else if uart_mcu_spec_config.operation == Syncen::SynchronousMode {
            bus.usart().cfg().modify(|_, w| w.syncen().synchronous_mode());

            if uart_mcu_spec_config.sync_mode_master_select == Syncmst::Master {
                bus.usart().cfg().modify(|_, w| w.syncmst().master());
            } else if uart_mcu_spec_config.sync_mode_master_select == Syncmst::Slave {
                bus.usart().cfg().modify(|_, w| w.syncmst().slave());
            }
        }

        if uart_mcu_spec_config.clock_polarity == Clkpol::RisingEdge {
            bus.usart().cfg().modify(|_, w| w.clkpol().rising_edge());
        } else if uart_mcu_spec_config.clock_polarity == Clkpol::FallingEdge {
            bus.usart().cfg().modify(|_, w| w.clkpol().falling_edge());
        }

        bus.usart().cfg().modify(|_, w| w.enable().enabled());
    }

    /// Deinitializes a USART instance.
    pub fn deinit(&self) -> Result<()> {
        // This function waits for TX complete, disables TX and RX, and disables the USART clock

        let bus = &self.bus;

        while bus.usart().stat().read().txidle().bit_is_clear() {
            // When 0, indicates that the transmitter is currently in the process of sending data.
        }

        // Disable interrupts
        bus.usart().fifointenclr().modify(|_, w| {
            w.txerr()
                .set_bit()
                .rxerr()
                .set_bit()
                .txlvl()
                .set_bit()
                .rxlvl()
                .set_bit()
        });

        //  Disable dma requests
        bus.usart()
            .fifocfg()
            .modify(|_, w| w.dmatx().clear_bit().dmarx().clear_bit());

        // Disable peripheral
        bus.usart().cfg().modify(|_, w| w.enable().disabled());

        Ok(())
    }

    /// Read RX data register using a blocking method.
    /// This function polls the RX register, waits for the RX register to be full or for RX FIFO to
    /// have data and read data from the TX register.
    /// Note for testing purpose : Blocking read API, that can receive a max of data of 8 bytes.
    /// The actual data expected to be received should be sent as "len"
    pub fn read_blocking(&self, buf: &mut [u8], len: u32) -> Result<()> {
        let bus = &self.bus;
        if len > 8 {
            return Err(GenericStatus::InvalidArgument);
        }

        // Check if rxFifo is not enabled
        if bus.usart().fifocfg().read().enablerx().is_disabled() {
            return Err(GenericStatus::Fail);
        } else {
            // rxfifo is enabled
            for i in 0..len {
                // loop until rxFifo has some data to read
                while bus.usart().fifostat().read().rxnotempty().bit_is_clear() {}

                // Now that there is some data in the rxFifo, read it
                // Let's verify the rxFifo status flags
                if bus.usart().fifostat().read().rxerr().bit_is_set() {
                    bus.usart().fifocfg().modify(|_, w| w.emptyrx().set_bit());
                    bus.usart().fifostat().modify(|_, w| w.rxerr().set_bit());
                    return Err(GenericStatus::UsartRxError);
                }

                // Save the receive status flag to check later.
                let rx_status = bus.usart().stat().read().bits();
                let mut generic_status = GenericStatus::Success;

                // clear all status flags

                //TODO: Note that bits 13,14 and 15 (FrameErrInt, ParityErrInt, ExNoiseErrInt) of uart::Stat reg is R/W1C, but in the  imxrt632s-pac, the read for these bits is not implemented..
                // Need to add the implementation for these bits in the pac file

                if rx_status & (1 << 14) != 0 {
                    //writing to it will clear the status since it is W1C
                    bus.usart().stat().modify(|_, w| w.parityerrint().set_bit());
                    generic_status = GenericStatus::UsartParityError;
                }
                if rx_status & (1 << 13) != 0 {
                    //writing to it will clear the status since it is W1C
                    bus.usart().stat().modify(|_, w| w.framerrint().set_bit());
                    generic_status = GenericStatus::UsartFramingError;
                }
                if rx_status & (1 << 15) != 0 {
                    //writing to it will clear the status since it is W1C
                    bus.usart().stat().modify(|_, w| w.rxnoiseint().set_bit());
                    generic_status = GenericStatus::UsartNoiseError;
                }

                if generic_status == GenericStatus::Success {
                    // read the data from the rxFifo
                    buf[i as usize] = bus.usart().fiford().read().rxdata().bits() as u8;
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
    pub fn write_blocking(&self, buf: &mut [u8], len: u32) -> Result<()> {
        let bus = &self.bus;
        // Check whether txFIFO is enabled
        if bus.usart().fifocfg().read().enabletx().is_disabled() {
            return Err(GenericStatus::Fail);
        } else {
            for i in 0..len {
                // Loop until txFIFO get some space for new data
                while bus.usart().fifostat().read().txnotfull().bit_is_clear() {}
                let x = buf[i as usize];
                bus.usart().fifowr().write(|w| unsafe { w.txdata().bits(x as u16) });
            }
            // Wait to finish transfer
            while bus.usart().stat().read().txidle().bit_is_clear() {}
        }
        Ok(())
    }
}

macro_rules! impl_uart_pin {
    ($piom_n:ident, $fn:ident, $fcn:expr) => {
        impl UartPin<$fcn> for crate::peripherals::$piom_n {
            fn as_txrx(&self) {
                // UM11147 table 299 pg 262+
                self.set_function(crate::iopctl::Function::$fn);
                self.set_drive_mode(DriveMode::PushPull);
                self.set_pull(Pull::None);
                self.set_slew_rate(SlewRate::Slow);
                self.set_drive_strength(DriveStrength::Normal);
                self.disable_analog_multiplex();
                self.enable_input_buffer();
            }
        }
    };
}

// Flexcomm0 Uart TX/Rx
impl_uart_pin!(PIO0_1, F1, 0); //Tx
impl_uart_pin!(PIO0_2, F1, 0); //Rx
impl_uart_pin!(PIO3_1, F5, 0); //Tx
impl_uart_pin!(PIO3_2, F5, 0); //Rx

// Flexcomm1 Uart TX/Rx
impl_uart_pin!(PIO0_8, F1, 1); //Tx
impl_uart_pin!(PIO0_9, F1, 1); //Rx
impl_uart_pin!(PIO7_26, F1, 1); //Tx
impl_uart_pin!(PIO7_27, F1, 1); //Rx

// Flexcomm2 Uart Tx/Rx
impl_uart_pin!(PIO0_15, F1, 2); //Tx
impl_uart_pin!(PIO0_16, F1, 2); //Rx
impl_uart_pin!(PIO7_30, F5, 2); //Tx
impl_uart_pin!(PIO7_31, F5, 2); //Rx

// Flexcomm3 Uart Tx/Rx
impl_uart_pin!(PIO0_22, F1, 3); //Tx
impl_uart_pin!(PIO0_23, F1, 3); //Rx

// Flexcomm4 Uart Tx/Rx
impl_uart_pin!(PIO0_29, F1, 4); //Tx
impl_uart_pin!(PIO0_30, F1, 4); //Rx

// Flexcomm5 Uart Tx/Rx
impl_uart_pin!(PIO1_4, F1, 5); //Tx
impl_uart_pin!(PIO1_5, F1, 5); //Rx
impl_uart_pin!(PIO3_16, F5, 5); //Tx
impl_uart_pin!(PIO3_17, F5, 5); //Rx

// Flexcomm6 Uart Tx/Rx
impl_uart_pin!(PIO3_26, F1, 6); //Tx
impl_uart_pin!(PIO3_27, F1, 6); //Rx

// Flexcomm7 Uart Tx/Rx
impl_uart_pin!(PIO4_1, F1, 7); //Tx
impl_uart_pin!(PIO4_2, F1, 7); //Rx
