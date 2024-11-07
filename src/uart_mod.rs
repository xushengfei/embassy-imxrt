use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::interrupt;
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::iopctl::*;
use crate::uart::Instance;
use crate::uart::{Error, Result, TransferError};
use crate::uart::{GeneralConfig, UartMcuSpecificConfig};
use sealed::Sealed;

type Baudrate = u32;

use mimxrt685s_pac as pac;
// Re-export SVD variants to allow user to directly set values.
use pac::usart0::cfg::Datalen;
/// Syncen : Sync/ Async mode selection
use pac::usart0::cfg::Syncen;
/// Syncmst : Sync master/slave mode selection (only applicable in sync mode)
use pac::usart0::cfg::Syncmst;
use pac::usart0::cfg::{Clkpol, Loop};
use pac::usart0::cfg::{Paritysel as Parity, Stoplen};
use pac::usart0::ctl::Cc;

/// UART driver.
pub struct Uart<'a, FC: Instance, M: Mode> {
    bus: crate::flexcomm::UsartBus<'a, FC>,
    tx: UartTx<'a, FC, M>,
    rx: UartRx<'a, FC, M>,
}

/// UART TX driver.
pub struct UartTx<'a, FC: Instance, M: Mode> {
    //tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    //bus: &'a crate::flexcomm::UsartBus<'a, FC>,
    phantom: PhantomData<(&'a mut FC, M)>,
}

/// UART RX driver.
pub struct UartRx<'a, FC: Instance, M: Mode> {
    // rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<(&'a mut FC, M)>,
}

impl<'a, FC: Instance, M: Mode> UartTx<'a, FC, M> {
    //fn new_inner(bus: &'a crate::flexcomm::UsartBus<'a, FC>) -> Self {
    fn new_inner() -> Self {
        Self {
            //bus,
            phantom: PhantomData,
        }
    }

    // TODO: Add more APIs
}

impl<'a, FC: Instance> UartTx<'a, FC, Blocking> {
    /// Transmit the provided buffer blocking execution until done.
    /// TODO: change the signature to : blocking_write(&mut self, buffer: &[u8]) -> Result<()>
    pub fn blocking_write(&self, bus: &crate::flexcomm::UsartBus<'a, FC>, buf: &mut [u8], len: u32) -> Result<()> {
        // Check whether txFIFO is enabled
        info!("Blocking_write is invoked");
        if bus.usart().fifocfg().read().enabletx().is_disabled() {
            return Err(Error::Fail);
        } else {
            for i in 0..len {
                // Loop until txFIFO get some space for new data
                while bus.usart().fifostat().read().txnotfull().bit_is_clear() {}
                let x = buf[i as usize];
                // SAFETY: unsafe only used for .bits()
                bus.usart().fifowr().write(|w| unsafe { w.txdata().bits(x as u16) });
            }
            // Wait to finish transfer
            while bus.usart().stat().read().txidle().bit_is_clear() {}
        }
        Ok(())
    }
}

/*
impl<'d, T: Instance> UartTx<'d, T, Async> {
    /// Write to UART TX from the provided buffer using DMA.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), TransferError> {
        // TODO: Start DMA transfer and call await.
        Ok(())
    }
}
    */

impl<'a, FC: Instance, M: Mode> UartRx<'a, FC, M> {
    fn new_inner(has_irq: bool) -> Self {
        // TODO: // disable all error interrupts initially
        // debug_assert_eq!(has_irq, rx_dma.is_some());
        Self { phantom: PhantomData }
    }
}

impl<'a, FC: Instance> UartRx<'a, FC, Blocking> {
    /// Read from UART RX blocking execution until done.
    /// TODO: change the func signature to blocking_read(&mut self, mut buffer: &mut [u8]) -> Result<(), TransferError>
    pub fn blocking_read(&self, bus: &crate::flexcomm::UsartBus<'a, FC>, buf: &mut [u8], len: u32) -> Result<()> {
        //let bus = &self.bus;
        info!("Blocking_read is invoked");

        // Check if rxFifo is not enabled
        if bus.usart().fifocfg().read().enablerx().is_disabled() {
            return Err(Error::Fail);
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
                    return Err(Error::Transfer(TransferError::UsartRxError));
                }

                let mut read_status = false; // false implies failure
                let mut generic_status = Error::Fail;

                // clear all status flags
                if bus.usart().stat().read().parityerrint().bit_is_set() {
                    bus.usart().stat().modify(|_, w| w.parityerrint().clear_bit_by_one());
                    generic_status = Error::Transfer(TransferError::UsartParityError);
                } else if bus.usart().stat().read().framerrint().bit_is_set() {
                    bus.usart().stat().modify(|_, w| w.framerrint().clear_bit_by_one());
                    generic_status = Error::Transfer(TransferError::UsartFramingError);
                } else if bus.usart().stat().read().rxnoiseint().bit_is_set() {
                    bus.usart().stat().modify(|_, w| w.rxnoiseint().clear_bit_by_one());
                    generic_status = Error::Transfer(TransferError::UsartNoiseError);
                } else {
                    // No error, proceed with read
                    read_status = true;
                }

                if read_status {
                    // read the data from the rxFifo
                    buf[i as usize] = bus.usart().fiford().read().rxdata().bits() as u8;
                } else {
                    return Err(generic_status);
                }
            }
        }

        Ok(())
    }
}

/*
impl<'d, FC: Instance> UartRx<'d, FC, Async> {
    /// Read from UART RX into the provided buffer.
    pub async fn read(&mut self, mut buffer: &mut [u8]) -> Result<(), TransferError> {
        // TODO: Start DMA transfer and call await.
        Ok(())
    }
}
*/

impl<'d, FC: Instance> Uart<'d, FC, Blocking> {
    /// Create a new UART without hardware flow control
    pub fn new_blocking(
        fc: impl Instance<P = FC> + 'd, //uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<FC>> + 'd,
        rx: impl Peripheral<P = impl RxPin<FC>> + 'd,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;

        into_ref!(tx, rx);
        tx.as_tx();
        rx.as_rx();

        let bus = crate::flexcomm::UsartBus::new_blocking(fc, clock)?;
        Self::new_inner(bus, false, general_config, mcu_spec_config)
    }

    /// Create a new UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts_blocking(
        fc: impl Instance<P = FC> + 'd, //uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<FC>> + 'd,
        rx: impl Peripheral<P = impl RxPin<FC>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<FC>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<FC>> + 'd,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;

        into_ref!(tx, rx, cts, rts);
        tx.as_tx();
        rx.as_rx();
        cts.as_cts();
        rts.as_rts();

        let bus = crate::flexcomm::UsartBus::new_blocking(fc, clock)?;
        Self::new_inner(bus, false, general_config, mcu_spec_config)
    }

    pub fn blocking_read(&self, buf: &mut [u8], len: u32) -> Result<()> {
        self.rx.blocking_read(&self.bus, buf, len)
    }
    pub fn blocking_write(&self, buf: &mut [u8], len: u32) -> Result<()> {
        self.tx.blocking_write(&self.bus, buf, len)
    }
}

impl<'d, FC: Instance> Uart<'d, FC, Async> {
    /// Create a new DMA enabled UART without hardware flow control
    pub fn new(
        fc: impl Instance<P = FC> + 'd,
        tx: impl Peripheral<P = impl TxPin<FC>> + 'd,
        rx: impl Peripheral<P = impl RxPin<FC>> + 'd,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;
        into_ref!(tx, rx);
        tx.as_tx();
        rx.as_rx();

        let bus = crate::flexcomm::UsartBus::new_async(fc, clock)?;
        Self::new_inner(bus, true, general_config, mcu_spec_config)
    }

    /// Create a new DMA enabled UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        fc: impl Instance<P = FC> + 'd, //uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<FC>> + 'd,
        rx: impl Peripheral<P = impl RxPin<FC>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<FC>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<FC>> + 'd,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;

        into_ref!(tx, rx, cts, rts);
        tx.as_tx();
        rx.as_rx();
        cts.as_cts();
        rts.as_rts();

        let bus = crate::flexcomm::UsartBus::new_async(fc, clock)?;

        Self::new_inner(bus, true, general_config, mcu_spec_config)
    }
}

impl<'a, FC: Instance, M: Mode> Uart<'a, FC, M> {
    fn new_inner(
        bus: crate::flexcomm::UsartBus<'a, FC>,
        has_irq: bool,
        general_config: GeneralConfig,
        mcu_spec_config: UartMcuSpecificConfig,
    ) -> Result<Self> {
        let this = Self {
            bus,
            tx: UartTx::new_inner(),
            rx: UartRx::new_inner(has_irq),
        };

        let _ = this.init(general_config, mcu_spec_config);

        Ok(this)
    }

    /// Split the Uart into a transmitter and receiver, which is particularly
    /// useful when having two tasks correlating to transmitting and receiving.
    pub fn split(self) -> (UartTx<'a, FC, M>, UartRx<'a, FC, M>) {
        (self.tx, self.rx)
    }

    /*fn getbus(&self) -> &'a crate::flexcomm::UsartBus<'a, FC> {
        &self.bus
    }*/

    fn init(&self, general_config: GeneralConfig, mcu_spec_config: UartMcuSpecificConfig) -> Result<()> {
        //tx.as_tx();
        self.set_uart_tx_fifo();

        //rx.as_rx();
        self.set_uart_rx_fifo();

        self.set_uart_baudrate(&general_config)?;

        self.set_uart_config(&general_config, &mcu_spec_config);

        Ok(())
    }

    fn get_fc_freq(&self) -> u32 {
        // Todo: Make it generic for any clock
        // Since the FC clock is hardcoded to Sfro, this freq is returned.
        // sfro : 0xf42400, //ffro: 0x2dc6c00
        0xf42400
    }

    fn set_uart_baudrate(&self, gen_config: &GeneralConfig) -> Result<()> {
        let bus = &self.bus;
        let baudrate_bps = gen_config.baudrate;
        let source_clock_hz = self.get_fc_freq(); // TODO: replace this with the call to flexcomm_getClkFreq()

        let mut best_diff: u32 = 0xFFFFFFFF;
        let mut best_osrval: u32 = 0xF;
        let mut best_brgval: u32 = 0xFFFFFFFF;
        let mut diff: u32;

        if baudrate_bps == 0 || source_clock_hz == 0 {
            return Err(Error::InvalidArgument);
        }

        // If synchronous master mode is enabled, only configure the BRG value.
        if bus.usart().cfg().read().syncen().is_synchronous_mode() {
            // Master
            if bus.usart().cfg().read().syncmst().is_master() {
                // Calculate the BRG value
                let mut brgval = source_clock_hz / baudrate_bps;
                brgval -= 1u32;
                // SAFETY: unsafe only used for .bits()
                bus.usart().brg().write(|w| unsafe { w.brgval().bits(brgval as u16) });
            }
        } else {
            // Smaller values of OSR can make the sampling position within a data bit less accurate and may
            // potentially cause more noise errors or incorrect data.
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
                return Err(Error::UsartBaudrateNotSupported);
            }

            // SAFETY: unsafe only used for .bits()
            bus.usart()
                .osr()
                .write(|w| unsafe { w.osrval().bits(best_osrval as u8) });
            // SAFETY: unsafe only used for .bits()
            bus.usart()
                .brg()
                .write(|w| unsafe { w.brgval().bits(best_brgval as u16) });
        }

        Ok(())
    }

    fn set_uart_tx_fifo(&self) {
        let bus = &self.bus;
        bus.usart()
            .fifocfg()
            .modify(|_, w| w.emptytx().set_bit().enabletx().enabled());

        // clear FIFO error
        bus.usart().fifostat().write(|w| w.txerr().set_bit());
    }

    fn set_uart_rx_fifo(&self) {
        let bus = &self.bus;
        bus.usart()
            .fifocfg()
            .modify(|_, w| w.emptyrx().set_bit().enablerx().enabled());

        // clear FIFO error
        bus.usart().fifostat().write(|w| w.rxerr().set_bit());
    }

    fn set_uart_config(&self, gen_config: &GeneralConfig, uart_mcu_spec_config: &UartMcuSpecificConfig) {
        let bus = &self.bus;
        bus.usart().cfg().write(|w| w.enable().disabled());

        // setting the uart data len
        match gen_config.data_bits {
            Datalen::Bit8 => bus.usart().cfg().modify(|_, w| w.datalen().bit_8()),
            Datalen::Bit7 => bus.usart().cfg().modify(|_, w| w.datalen().bit_7()),
            Datalen::Bit9 => bus.usart().cfg().modify(|_, w| w.datalen().bit_9()),
        }

        // setting the uart stop bits
        match gen_config.stop_bits {
            Stoplen::Bit1 => bus.usart().cfg().modify(|_, w| w.stoplen().bit_1()),
            Stoplen::Bits2 => bus.usart().cfg().modify(|_, w| w.stoplen().bits_2()),
        }

        // setting the uart parity
        match gen_config.parity {
            Parity::NoParity => bus.usart().cfg().modify(|_, w| w.paritysel().no_parity()),
            Parity::EvenParity => bus.usart().cfg().modify(|_, w| w.paritysel().even_parity()),
            Parity::OddParity => bus.usart().cfg().modify(|_, w| w.paritysel().odd_parity()),
        }

        // setting mcu specific uart config
        match uart_mcu_spec_config.loopback_mode {
            Loop::Normal => bus.usart().cfg().modify(|_, w| w.loop_().normal()),
            Loop::Loopback => bus.usart().cfg().modify(|_, w| w.loop_().loopback()),
        }

        match uart_mcu_spec_config.operation {
            Syncen::AsynchronousMode => bus.usart().cfg().modify(|_, w| w.syncen().asynchronous_mode()),
            Syncen::SynchronousMode => {
                bus.usart().cfg().modify(|_, w| w.syncen().synchronous_mode());
                match uart_mcu_spec_config.sync_mode_master_select {
                    Syncmst::Master => bus.usart().cfg().modify(|_, w| w.syncmst().master()),
                    Syncmst::Slave => bus.usart().cfg().modify(|_, w| w.syncmst().slave()),
                }
            }
        }

        match uart_mcu_spec_config.clock_polarity {
            Clkpol::RisingEdge => bus.usart().cfg().modify(|_, w| w.clkpol().rising_edge()),
            Clkpol::FallingEdge => bus.usart().cfg().modify(|_, w| w.clkpol().falling_edge()),
        }

        bus.usart().cfg().modify(|_, w| w.enable().enabled());
    }
}

mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

/// UART mode.
#[allow(private_bounds)]
pub trait Mode: Sealed {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl Sealed for $name {}
        impl Mode for $name {}
    };
}

impl<T: IopctlPin> sealed::Sealed for T {}

/// Blocking mode.
pub struct Blocking;
/// Async mode.
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

/// Trait for TX pins.
pub trait TxPin<T: Instance>: sealed::Sealed + crate::gpio::GpioPin {
    /// convert the pin to appropriate function for Uart Tx  usage
    fn as_tx(&self);
}
/// Trait for RX pins.
pub trait RxPin<T: Instance>: sealed::Sealed + crate::gpio::GpioPin {
    /// convert the pin to appropriate function for Uart Rx  usage
    fn as_rx(&self);
}
/// Trait for Clear To Send (CTS) pins.
pub trait CtsPin<T: Instance>: sealed::Sealed + crate::gpio::GpioPin {
    /// convert the pin to appropriate function for Uart Cts  usage
    fn as_cts(&self);
}
/// Trait for Request To Send (RTS) pins.
pub trait RtsPin<T: Instance>: sealed::Sealed + crate::gpio::GpioPin {
    /// convert the pin to appropriate function for Uart rts  usage
    fn as_rts(&self);
}

macro_rules! impl_uart_rx {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        //impl UartRx<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
        impl RxPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_rx(&self) {
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

macro_rules! impl_uart_tx {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        //impl UartRx<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
        impl TxPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_tx(&self) {
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

macro_rules! impl_uart_rts {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        //impl UartRx<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
        impl RtsPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_rts(&self) {
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

macro_rules! impl_uart_cts {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        //impl UartRx<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
        impl CtsPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_cts(&self) {
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
impl_uart_tx!(PIO0_1, F1, FLEXCOMM0); //Tx
impl_uart_rx!(PIO0_2, F1, FLEXCOMM0); //Rx
impl_uart_cts!(PIO0_3, F1, FLEXCOMM0); //Cts
impl_uart_rts!(PIO0_4, F1, FLEXCOMM0); //Rts

// Flexcomm1 Uart TX/Rx
impl_uart_tx!(PIO0_8, F1, FLEXCOMM1); //Tx
impl_uart_rx!(PIO0_9, F1, FLEXCOMM1); //Rx
impl_uart_cts!(PIO0_10, F1, FLEXCOMM1); //Cts
impl_uart_rts!(PIO0_11, F1, FLEXCOMM1); //Rts

// Flexcomm2 Uart TX/Rx
impl_uart_tx!(PIO0_15, F1, FLEXCOMM2); //Tx
impl_uart_rx!(PIO0_16, F1, FLEXCOMM2); //Rx
impl_uart_cts!(PIO0_17, F1, FLEXCOMM2); //Cts
impl_uart_rts!(PIO0_18, F1, FLEXCOMM2); //Rts
