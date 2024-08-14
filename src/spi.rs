//! Serial Peripheral Interface
use core::marker::PhantomData;

use embassy_futures::join::join;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::{pac, peripherals, Peripheral};

//use crate::pac::gpio::{AnyPin, Pin as GpioPin, SealedPin as _};
// todo: AnyPin temporary placeholder until gpio apis available
pub struct AnyPin {
    pin_port: u8,
}

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

impl<'d, T: Instance, M: Mode> Spi<'d, T, M> {
    fn new_inner(
        inner: impl Peripheral<P = T> + 'd,
        clk: Option<PeripheralRef<'d, AnyPin>>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        cs: Option<PeripheralRef<'d, AnyPin>>,
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

impl_instance!(SPIO0, Spi0, 16, 17);

/// CLK pin.
pub trait ClkPin<T: Instance>: GpioPin {}
/// CS pin.
pub trait CsPin<T: Instance>: GpioPin {}
/// MOSI pin.
pub trait MosiPin<T: Instance>: GpioPin {}
/// MISO pin.
pub trait MisoPin<T: Instance>: GpioPin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, SPI0, MisoPin);
impl_pin!(PIN_1, SPI0, CsPin);
impl_pin!(PIN_2, SPI0, ClkPin);
impl_pin!(PIN_3, SPI0, MosiPin);
impl_pin!(PIN_4, SPI0, MisoPin);
impl_pin!(PIN_5, SPI0, CsPin);
impl_pin!(PIN_6, SPI0, ClkPin);
impl_pin!(PIN_7, SPI0, MosiPin);
impl_pin!(PIN_8, SPI1, MisoPin);
impl_pin!(PIN_9, SPI1, CsPin);
impl_pin!(PIN_10, SPI1, ClkPin);
impl_pin!(PIN_11, SPI1, MosiPin);
impl_pin!(PIN_12, SPI1, MisoPin);
impl_pin!(PIN_13, SPI1, CsPin);
impl_pin!(PIN_14, SPI1, ClkPin);
impl_pin!(PIN_15, SPI1, MosiPin);
impl_pin!(PIN_16, SPI0, MisoPin);
impl_pin!(PIN_17, SPI0, CsPin);
impl_pin!(PIN_18, SPI0, ClkPin);
impl_pin!(PIN_19, SPI0, MosiPin);
impl_pin!(PIN_20, SPI0, MisoPin);
impl_pin!(PIN_21, SPI0, CsPin);
impl_pin!(PIN_22, SPI0, ClkPin);
impl_pin!(PIN_23, SPI0, MosiPin);
impl_pin!(PIN_24, SPI1, MisoPin);
impl_pin!(PIN_25, SPI1, CsPin);
impl_pin!(PIN_26, SPI1, ClkPin);
impl_pin!(PIN_27, SPI1, MosiPin);
impl_pin!(PIN_28, SPI1, MisoPin);
impl_pin!(PIN_29, SPI1, CsPin);

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
