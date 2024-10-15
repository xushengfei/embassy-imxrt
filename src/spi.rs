//! Serial Peripheral Interface
use core::marker::PhantomData;

use embassy_futures::join::join;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::{pac, peripherals, Peripheral};

use crate::iopctl::IopctlPin as Pin;

/// SPI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No detailed errors specified
}

/// SPI configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// Frequency.
    pub frequency: u32,
    /// Phase.
    pub phase: Phase,
    /// Polarity.
    pub polarity: Polarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            phase: Phase::CaptureOnFirstTransition,
            polarity: Polarity::IdleLow,
        }
    }
}

/// SPI driver.
pub struct Spi<'d, T: Instance, M: Mode> {
    inner: PeripheralRef<'d, T>,
    phantom: PhantomData<(&'d mut T, M)>,
}

fn calc_prescs(freq: u32) -> (u8, u8) {
    todo!();
}

impl<'d, FC: Instance, M: Mode> Spi<'d, T, M> {
    fn new_inner(
        inner: impl Peripheral<P = FC> + 'd,
        clk: impl SckPin<FC> + 'd,
        mosi: impl MosiPin<FC> + 'd,
        miso: impl MisoPin<FC> + 'd,
        ssel0: impl SselPin<FC> + 'd,
        ssel1: impl SselPin<FC> + 'd,
        ssel2: impl SselPin<FC> + 'd,
        ssel3: impl SselPin<FC> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(inner);

        let p = inner.regs();
        // todo: calculate and write prescaler

        // todo: set data size, polarity, phase, post divider

        // todo: enable dma, if async mode

        // todo: configure gpio for spi

        // todo: enable spi

        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Write data to SPI blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        for &b in data {
            while !p.stat().read().mstidle() {}
            p.dr().write(|w| w.set_data(b as _));
            while !p.sr().read().rne() {}
            let _ = p.dr().read();
        }
        self.flush()?;
        Ok(())
    }

    /// Transfer data in place to SPI blocking execution until done.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        for b in data {
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(*b as _));
            while !p.sr().read().rne() {}
            *b = p.dr().read().data() as u8;
        }
        self.flush()?;
        Ok(())
    }

    /// Read data from SPI blocking execution until done.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        for b in data {
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(0));
            while !p.sr().read().rne() {}
            *b = p.dr().read().data() as u8;
        }
        self.flush()?;
        Ok(())
    }

    /// Transfer data to SPI blocking execution until done.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        let len = read.len().max(write.len());
        for i in 0..len {
            let wb = write.get(i).copied().unwrap_or(0);
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(wb as _));
            while !p.sr().read().rne() {}
            let rb = p.dr().read().data() as u8;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }
        self.flush()?;
        Ok(())
    }

    /// Block execution until SPI is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        let p = self.inner.regs();
        while p.sr().read().bsy() {}
        Ok(())
    }

    /// Set SPI frequency.
    pub fn set_frequency(&mut self, freq: u32) {
        let (presc, postdiv) = calc_prescs(freq);
        let p = self.inner.regs();
        // disable
        p.cr1().write(|w| w.set_sse(false));

        // change stuff
        p.cpsr().write(|w| w.set_cpsdvsr(presc));
        p.cr0().modify(|w| {
            w.set_scr(postdiv);
        });

        // enable
        p.cr1().write(|w| w.set_sse(true));
    }
}

impl<'d, T: Instance> Spi<'d, T, Blocking> {
    /// Create an SPI driver in blocking mode.
    pub fn new_blocking(
        inner: impl Peripheral<P = T> + 'd,
        clk: impl Peripheral<P = impl ClkPin<T> + 'd> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T> + 'd> + 'd,
        miso: impl Peripheral<P = impl MisoPin<T> + 'd> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, mosi, miso);
        Self::new_inner(
            inner,
            Some(clk.map_into()),
            Some(mosi.map_into()),
            Some(miso.map_into()),
            None,
            config,
        )
    }
}

trait SealedMode {}

trait SealedInstance {
    const TX_DREQ: u8;
    const RX_DREQ: u8;

    fn regs(&self) -> crate::pac::spi0::RegisterBlock;
}

/// Mode.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

/// SPI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl SealedInstance for peripherals::$type {
            const TX_DREQ: u8 = $tx_dreq;
            const RX_DREQ: u8 = $rx_dreq;

            fn regs(&self) -> crate::pac::spi0::RegisterBlock {
                pac::$type
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(FLEXCOMM0, Spi0, 16, 17);

mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

impl<T: Pin> sealed::Sealed for T {}

/// io configuration trait for Mosi
pub trait MosiPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for mosi usage
    fn as_mosi(&self, pull: crate::iopctl::Pull);
}

/// io configuration trait for Miso
pub trait MisoPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for miso usage
    fn as_miso(&self, pull: crate::iopctl::Pull);
}

/// io configuration trait for Sck (serial clock)
pub trait SckPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for sck usage
    fn as_sck(&self, pull: crate::iopctl::Pull);
}

/// io configuration trait for Ssel n (chip select n)
pub trait SselPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for ssel usage
    fn as_ssel(&self, pull: crate::iopctl::Pull);
}

// flexcomm <-> Pin function map
macro_rules! impl_miso {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl MosiPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_mosi(&self, pull: crate::iopctl::Pull) {
                // UM11147 table 299 pg 262+
                self.set_pull(pull)
                    .set_slew_rate(crate::gpio::SlewRate::Standard)
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
                    .set_drive_mode(crate::gpio::DriveMode::PushPull)
                    .set_input_polarity(crate::gpio::Polarity::ActiveHigh)
                    .enable_input_buffer()
                    .set_function(crate::iopctl::Function::$fn);
            }
        }
    };
}
macro_rules! impl_mosi {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl MisoPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_miso(&self, pull: crate::iopctl::Pull) {
                // UM11147 table 299 pg 262+
                self.set_pull(pull)
                    .set_slew_rate(crate::gpio::SlewRate::Standard)
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
                    .set_drive_mode(crate::gpio::DriveMode::PushPull)
                    .set_input_polarity(crate::gpio::Polarity::ActiveHigh)
                    .enable_input_buffer()
                    .set_function(crate::iopctl::Function::$fn);
            }
        }
    };
}
macro_rules! impl_sck {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl SckPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_sck(&self, pull: crate::iopctl::Pull) {
                // UM11147 table 299 pg 262+
                self.set_pull(pull)
                    .set_slew_rate(crate::gpio::SlewRate::Standard)
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
                    .set_drive_mode(crate::gpio::DriveMode::PushPull)
                    .set_input_polarity(crate::gpio::Polarity::ActiveHigh)
                    .enable_input_buffer()
                    .set_function(crate::iopctl::Function::$fn);
            }
        }
    };
}
macro_rules! impl_ssel {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl SselPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_ssel(&self, pull: crate::iopctl::Pull) {
                // UM11147 table 299 pg 262+
                self.set_pull(pull)
                    .set_slew_rate(crate::gpio::SlewRate::Standard)
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
                    .set_drive_mode(crate::gpio::DriveMode::PushPull)
                    .set_input_polarity(crate::gpio::Polarity::ActiveHigh)
                    .enable_input_buffer()
                    .set_function(crate::iopctl::Function::$fn);
            }
        }
    };
}

/// Flexcomm0 SPI GPIOs -
impl_miso!(PIO0_1, F1, FLEXCOMM0);
impl_mosi!(PIO0_2, F1, FLEXCOMM0);
impl_sck!(PIO0_0, F1, FLEXCOMM0);
impl_ssel!(PIO0_3, F1, FLEXCOMM0);

macro_rules! impl_mode {
    ($name:ident) => {
        impl SealedMode for $name {}
        impl Mode for $name {}
    };
}

/// Blocking mode.
pub struct Blocking;

impl_mode!(Blocking);

// ====================

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, T, M> {
    type Error = Error;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.blocking_transfer_in_place(words)?;
        Ok(words)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, T, M> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }
}

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {}
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::ErrorType for Spi<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::SpiBus<u8> for Spi<'d, T, M> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(words, &[])
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }
}
