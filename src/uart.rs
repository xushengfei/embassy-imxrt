//!Universal Asynchronous Receiver Transmitter (UART) driver.
//!

#![macro_use]

use core::marker::PhantomData;

use crate::iopctl::*;
use crate::pac::usart1;
use crate::peripherals;
use crate::uart_setting::Flexcomm;
use embassy_hal_internal::{impl_peripheral, into_ref, Peripheral, PeripheralRef};
use mimxrt685s_pac as pac;
use pac::usart1::RegisterBlock;

// Re-export SVD variants to allow user to directly set values.
pub use pac::usart1::cfg::Datalen;
pub use pac::usart1::cfg::Paritysel as Parity;
pub use pac::usart1::cfg::Stoplen;
pub use u32 as Baudrate;

pub use pac::usart1::cfg::Clkpol;
pub use pac::usart1::cfg::Loop;
/// Syncen : Sync/ Async mode selection
pub use pac::usart1::cfg::Syncen;
/// Syncmst : Sync master/slave mode selection (only applicable in sync mode)
pub use pac::usart1::cfg::Syncmst;
pub use pac::usart1::ctl::Cc;

/// Todo: Will be used when the uart is fully implemented - both tx and rx. Right now only Rx is implemented
pub use pac::usart1::fifocfg::Enablerx;
pub use pac::usart1::fifocfg::Enabletx;

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

/// UartRxTx struct to hold the uart configuration
pub struct UartRxTx {
    pub baudrate: Baudrate,
    pub data_bits: Datalen,
    pub parity: Parity,
    pub stop_bits: Stoplen,
    pub flexcomm_freq: u32,
}

#[derive(Clone, Copy)]
pub struct UartMcuSpecific {
    pub clock_polarity: Clkpol,
    /// Sync/ Async operation selection
    pub operation: Syncen,
    /// Sync master/slave mode selection (only applicable in sync mode)
    pub sync_mode_master_select: Syncmst,
    /// USART continuous Clock generation enable in synchronous master mode.
    pub continuous_clock: Cc,
    pub loopback_mode: Loop,
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

impl UartRxTx {
    pub fn new() -> Self {
        UartRxTx {
            baudrate: 115200,
            data_bits: Datalen::Bit8,
            parity: Parity::NoParity,
            stop_bits: Stoplen::Bit1,
            flexcomm_freq: 0x2dc6c00, //0x2dc6c00, //20000000
        }
    }
    /// Exposing a method to access reg internally with the assumption that only the uart0 is being used
    fn reg(&self) -> &'static pac::usart1::RegisterBlock {
        unsafe { &*(pac::Usart1::ptr() as *const pac::usart1::RegisterBlock) }
    }

    /// Initializes a USART instance with user configuration structure and peripheral clock.
    /// Use this API to prog all the registers for the uart0 - assuming flexcomm0, clocks, ioctl are already set
    pub fn init(&self) {
        let default_mcu_specific_uart_config: UartMcuSpecific = UartMcuSpecific {
            clock_polarity: Clkpol::FallingEdge,
            operation: Syncen::AsynchronousMode,
            sync_mode_master_select: Syncmst::Slave,
            continuous_clock: Cc::ClockOnCharacter,
            loopback_mode: Loop::Normal,
        };

        Flexcomm::new().init();
        self.configure_pins();
        // TODO: Add if enableTx, Rx before calling self.set_uart_rx_fifo(), self.set_uart_tx_fifo();
        self.set_uart_tx_fifo();
        self.set_uart_rx_fifo();
        self.set_uart_config(default_mcu_specific_uart_config);
        self.set_uart_baudrate();

        // Setting continuous Clock configuration. used for synchronous master mode.
        self.enable_continuous_clock(default_mcu_specific_uart_config.continuous_clock);
    }

    /// Deinitializes a USART instance.
    pub fn deinit(&self) {
        // This function waits for TX complete, disables TX and RX, and disables the USART clock

        while self.reg().stat().read().txidle().bit_is_clear() {
            // When 0, indicates that the transmitter is currently in the process of sending data.
        }

        // Disable interrupts
        /*self.reg().fifointenclr().write(|w| w.txerr().set_bit());
        self.reg().fifointenclr().write(|w| w.rxerr().set_bit());
        self.reg().fifointenclr().write(|w| w.txlvl().set_bit());
        self.reg().fifointenclr().write(|w| w.rxlvl().set_bit());*/
        self.reg().fifointenclr().modify(|_, w| {
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
        /*self.reg().fifocfg().write(|w| w.dmatx().clear_bit());
        self.reg().fifocfg().write(|w| w.dmarx().clear_bit());*/
        self.reg()
            .fifocfg()
            .modify(|_, w| w.dmatx().clear_bit().dmarx().clear_bit());

        // Disable peripheral
        //self.reg().cfg().write(|w| w.enable().disabled());
        self.reg().cfg().modify(|_, w| w.enable().disabled());
    }

    /// Read RX data register using a blocking method.
    /// This function polls the RX register, waits for the RX register to be full or for RX FIFO to
    /// have data and read data from the TX register.
    /// Note for testing purpose : Blocking read API, that can receive a max of data of 8 bytes.
    /// The actual data expected to be received should be sent as "len"
    pub fn read_blocking(&self, buf: &mut [u8], len: u32) -> GenericStatus {
        if len > 8 {
            return GenericStatus::InvalidArgument;
        }

        // Check if rxFifo is not enabled
        if self.reg().fifocfg().read().enablerx().is_disabled() {
            return GenericStatus::Fail;
        } else {
            // rxfifo is enabled
            for i in 0..len {
                // Add to check to see of txfifo is empty (it should not be)
                if self.reg().fifostat().read().txempty().bit_is_set() {
                    info!("Error: TX FIFO is empty");
                } else {
                    info!("Info: TX FIFO is not empty");
                }
                // loop until rxFifo has some data to read
                while self.reg().fifostat().read().rxnotempty().bit_is_clear() {}

                // Now that there is some data in the rxFifo, read it
                // Let's verify the rxFifo status flags
                if self.reg().fifostat().read().rxerr().bit_is_set() {
                    //self.reg().fifocfg().write(|w| w.emptyrx().set_bit());
                    //self.reg().fifostat().write(|w| w.rxerr().set_bit());
                    self.reg().fifocfg().modify(|_, w| w.emptyrx().set_bit());
                    self.reg().fifostat().modify(|_, w| w.rxerr().set_bit());
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
                    //self.reg().stat().write(|w| w.parityerrint().set_bit());
                    self.reg().stat().modify(|_, w| w.parityerrint().set_bit());
                    generic_status = GenericStatus::USART_ParityError;
                }
                if rx_status & (1 << 13) != 0 {
                    //writing to it will clear the status since it is W1C
                    //self.reg().stat().write(|w| w.framerrint().set_bit());
                    self.reg().stat().modify(|_, w| w.framerrint().set_bit());
                    generic_status = GenericStatus::USART_FramingError;
                }
                if rx_status & (1 << 15) != 0 {
                    //writing to it will clear the status since it is W1C
                    //self.reg().stat().write(|w| w.rxnoiseint().set_bit());
                    self.reg().stat().modify(|_, w| w.rxnoiseint().set_bit());
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

    /// Writes to the TX register using a blocking method.
    /// This function polls the TX register, waits for the TX register to be empty or for the TX FIFO
    /// to have room and writes data to the TX buffer.
    /// Note for testing purpose : Blocking write API, that can send a max of data of 8 bytes.
    /// The actual data expected to be sent should be sent as "len"
    pub fn write_blocking(&self, buf: &mut [u8], len: u32) -> GenericStatus {
        // Check whether txFIFO is enabled
        if self.reg().fifocfg().read().enabletx().is_disabled() {
            return GenericStatus::Fail;
        } else {
            for i in 0..len {
                // Loop until txFIFO get some space for new data
                while self.reg().fifostat().read().txnotfull().bit_is_clear() {}
                let mut x = buf[i as usize];
                self.reg().fifowr().write(|w| unsafe { w.txdata().bits(x as u16) });
            }
            // Wait to finish transfer
            while self.reg().stat().read().txidle().bit_is_clear() {}
        }
        return GenericStatus::Success;
    }

    fn set_uart_config(&self, uart_mcu_config: UartMcuSpecific) {
        // setting the uart data len
        if self.data_bits == Datalen::Bit8 {
            //self.reg().cfg().write(|w| w.datalen().bit_8());
            self.reg().cfg().modify(|_, w| w.datalen().bit_8());
        } else if self.data_bits == Datalen::Bit7 {
            //self.reg().cfg().write(|w| w.datalen().bit_7());
            self.reg().cfg().modify(|_, w| w.datalen().bit_7());
        } else if self.data_bits == Datalen::Bit9 {
            //self.reg().cfg().write(|w| w.datalen().bit_9());
            self.reg().cfg().modify(|_, w| w.datalen().bit_9());
        }

        //setting the uart stop bits
        if self.stop_bits == Stoplen::Bit1 {
            //self.reg().cfg().write(|w| w.stoplen().bit_1());
            self.reg().cfg().modify(|_, w| w.stoplen().bit_1());
        } else if self.stop_bits == Stoplen::Bits2 {
            //self.reg().cfg().write(|w| w.stoplen().bits_2());
            self.reg().cfg().modify(|_, w| w.stoplen().bits_2());
        }

        //setting the uart parity
        if self.parity == Parity::NoParity {
            //self.reg().cfg().write(|w| w.paritysel().no_parity());
            self.reg().cfg().modify(|_, w| w.paritysel().no_parity());
        } else if self.parity == Parity::EvenParity {
            //self.reg().cfg().write(|w| w.paritysel().even_parity());
            self.reg().cfg().modify(|_, w| w.paritysel().even_parity());
        } else if self.parity == Parity::OddParity {
            //self.reg().cfg().write(|w| w.paritysel().odd_parity());
            self.reg().cfg().modify(|_, w| w.paritysel().odd_parity());
        }

        // setting mcu specific uart config
        if uart_mcu_config.loopback_mode == Loop::Normal {
            //self.reg().cfg().write(|w| w.loop_().normal());
            self.reg().cfg().modify(|_, w| w.loop_().normal());
        } else if uart_mcu_config.loopback_mode == Loop::Loopback {
            //self.reg().cfg().write(|w| w.loop_().loopback());
            self.reg().cfg().modify(|_, w| w.loop_().loopback());
        }

        if uart_mcu_config.operation == Syncen::AsynchronousMode {
            //self.reg().cfg().write(|w| w.syncen().asynchronous_mode());
            self.reg().cfg().modify(|_, w| w.syncen().asynchronous_mode());
        } else if uart_mcu_config.operation == Syncen::SynchronousMode {
            //self.reg().cfg().write(|w| w.syncen().synchronous_mode());
            self.reg().cfg().modify(|_, w| w.syncen().synchronous_mode());

            if uart_mcu_config.sync_mode_master_select == Syncmst::Master {
                //self.reg().cfg().write(|w| w.syncmst().master());
                self.reg().cfg().modify(|_, w| w.syncmst().master());
            } else if uart_mcu_config.sync_mode_master_select == Syncmst::Slave {
                //self.reg().cfg().write(|w| w.syncmst().slave());
                self.reg().cfg().modify(|_, w| w.syncmst().slave());
            }
        }

        if uart_mcu_config.clock_polarity == Clkpol::RisingEdge {
            //self.reg().cfg().write(|w| w.clkpol().rising_edge());
            self.reg().cfg().modify(|_, w| w.clkpol().rising_edge());
        } else if uart_mcu_config.clock_polarity == Clkpol::FallingEdge {
            //self.reg().cfg().write(|w| w.clkpol().falling_edge());
            self.reg().cfg().modify(|_, w| w.clkpol().falling_edge());
        }

        // Now enable the uart
        //self.reg().cfg().write(|w| w.enable().enabled());
        self.reg().cfg().modify(|_, w| w.enable().enabled());
    }

    fn set_uart_rx_fifo(&self) {
        // Todo : Add condition to check if (enableRx){}
        // The setting below needs to be encapsulated in a condition if (enableRx)
        // setting the rx fifo
        /*self.reg().fifocfg().write(|w| w.enablerx().enabled());
        self.reg().fifocfg().write(|w| w.emptyrx().set_bit());*/
        self.reg()
            .fifocfg()
            .modify(|_, w| w.emptyrx().set_bit().enablerx().enabled());

        if self.reg().fifocfg().read().enablerx().bit_is_clear() {
            info!("Error: RX FIFO is not enabled");
        } else {
            info!("Info: RX FIFO is enabled");
        }
        // Todo: Add code for setting Fifo trigger register. Refer to USART_Init() in fsl_uart.c
        //  setup trigger level
        //base->FIFOTRIG &= ~(USART_FIFOTRIG_RXLVL_MASK);
        //base->FIFOTRIG |= USART_FIFOTRIG_RXLVL(config->rxWatermark);
        /* enable trigger interrupt */
        //base->FIFOTRIG |= USART_FIFOTRIG_RXLVLENA_MASK;
        unsafe {
            self.reg()
                .fifotrig()
                .modify(|_, w| w.rxlvl().bits(0).rxlvlena().set_bit());
        }
    }

    fn set_uart_tx_fifo(&self) {
        // Todo : Add condition to check if (enableTx){}
        // The setting below needs to be encapsulated in a condition if (enableTx)
        // setting the Tx fifo

        // empty and enable txFIFO
        /*self.reg().fifocfg().write(|w| w.enabletx().enabled());
        self.reg().fifocfg().write(|w| w.emptytx().set_bit());
        */
        self.reg()
            .fifocfg()
            .modify(|_, w| w.emptytx().set_bit().enabletx().enabled());

        if self.reg().fifocfg().read().enabletx().bit_is_clear() {
            info!("Error: TX FIFO is not enabled");
        } else {
            info!("Info: TX FIFO is enabled");
        }

        //TODO for later
        /*
        //setup trigger level
        base->FIFOTRIG &= ~(USART_FIFOTRIG_TXLVL_MASK);
        base->FIFOTRIG |= USART_FIFOTRIG_TXLVL(config->txWatermark);
        // enable trigger interrupt
        base->FIFOTRIG |= USART_FIFOTRIG_TXLVLENA_MASK;
         */
        unsafe {
            self.reg()
                .fifotrig()
                .modify(|_, w| w.txlvl().bits(1).txlvlena().set_bit());
        }
    }

    fn set_uart_baudrate(&self) -> GenericStatus {
        let baudrate_bps = self.baudrate;
        let source_clock_hz = self.flexcomm_freq; // TODO: replace this with the call to flexcomm_getClkFreq()

        let mut best_diff: u32 = 0xFFFFFFFF;
        let mut best_osrval: u32 = 0xF;
        let mut best_brgval: u32 = 0xFFFFFFFF;
        let mut osrval: u32 = 0;
        let mut brgval: u32 = 0;
        let mut diff: u32 = 0;
        let mut baudrate: u32 = 0;

        if baudrate_bps == 0 || source_clock_hz == 0 {
            return GenericStatus::InvalidArgument;
        }

        //If synchronous master mode is enabled, only configure the BRG value.
        if self.reg().cfg().read().syncen().is_synchronous_mode() {
            // Master
            if self.reg().cfg().read().syncmst().is_master() {
                // Calculate the BRG value
                brgval = source_clock_hz / baudrate_bps;
                brgval = brgval - 1u32;
                unsafe { self.reg().brg().write(|w| w.brgval().bits(brgval as u16)) };
            }
        } else {
            //Smaller values of OSR can make the sampling position within a data bit less accurate and may
            //potentially cause more noise errors or incorrect data.
            for osrval in (8..=best_osrval).rev() {
                brgval = (((source_clock_hz * 10u32) / ((osrval + 1u32) * baudrate_bps)) - 5u32) / 10u32;
                if brgval > 0xFFFFu32 {
                    continue;
                }
                // Calculate the baud rate based on the BRG value
                baudrate = source_clock_hz / ((osrval + 1u32) * (brgval + 1u32));

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
                return GenericStatus::USART_BaudrateNotSupport;
            }

            unsafe {
                self.reg().osr().write(|w| w.osrval().bits(best_osrval as u8));
                self.reg().brg().write(|w| w.brgval().bits(best_brgval as u16));
            }
        }

        GenericStatus::Success
    }

    fn enable_continuous_clock(&self, continuous_clock: Cc) {
        if continuous_clock == Cc::ClockOnCharacter {
            //self.reg().ctl().write(|w| w.cc().clock_on_character());
            self.reg().ctl().modify(|_, w| w.cc().clock_on_character());
        } else if continuous_clock == Cc::ContinousClock {
            //self.reg().ctl().write(|w| w.cc().continous_clock());
            self.reg().ctl().modify(|_, w| w.cc().continous_clock());
        }
    }

    /// This function configures the uart pins for testing purpose.
    /// Note: this is not the correct way of calling the ioctl. This is just for testing purpose.
    fn configure_pins(&self) {
        let pin = unsafe { crate::peripherals::PIO0_8::steal() }; // Host uart tx
        pin.set_function(Function::F1);
        pin.set_drive_mode(DriveMode::PushPull);
        pin.set_pull(Pull::None);
        pin.set_slew_rate(SlewRate::Slow);
        pin.set_drive_strength(DriveStrength::Normal);
        pin.disable_analog_multiplex();
        pin.enable_input_buffer();

        let pin = unsafe { crate::peripherals::PIO0_9::steal() }; // Host uart rx
        pin.set_function(Function::F1);
        pin.set_drive_mode(DriveMode::PushPull);
        pin.set_pull(Pull::None);
        pin.set_slew_rate(SlewRate::Slow);
        pin.set_drive_strength(DriveStrength::Normal);
        pin.disable_analog_multiplex();
        pin.enable_input_buffer();

        /*
        let pin = unsafe { crate::peripherals::PIO3_3::steal() }; // Host uart cts
        pin.set_function(Function::F5);

        let pin = unsafe { crate::peripherals::PIO3_4::steal() }; // Host uart rts
        pin.set_function(Function::F5);
        */
    }
}
