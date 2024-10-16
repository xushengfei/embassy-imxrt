//! Serial Peripheral Interface
use core::marker::PhantomData;

use embassy_futures::join::join;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::{pac, peripherals, Peripheral};

use crate::iopctl::IopctlPin as Pin;
use sealed::Sealed;

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

mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

impl<T: Pin> sealed::Sealed for T {}

/// shared functions between master and slave operation
#[allow(private_bounds)]
pub trait Instance: crate::flexcomm::SpiPeripheral {}
impl Instance for crate::peripherals::FLEXCOMM0 {}
impl Instance for crate::peripherals::FLEXCOMM1 {}
impl Instance for crate::peripherals::FLEXCOMM2 {}
impl Instance for crate::peripherals::FLEXCOMM3 {}
impl Instance for crate::peripherals::FLEXCOMM4 {}
impl Instance for crate::peripherals::FLEXCOMM5 {}
impl Instance for crate::peripherals::FLEXCOMM6 {}
impl Instance for crate::peripherals::FLEXCOMM7 {}

/// io configuration trait for Mosi
pub trait MosiPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for mosi usage
    fn as_mosi(&self);
}

/// io configuration trait for Miso
pub trait MisoPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for miso usage
    fn as_miso(&self);
}

/// io configuration trait for Sck (serial clock)
pub trait SckPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for sck usage
    fn as_sck(&self);
}

/// io configuration trait for Ssel n (chip select n)
pub trait SselPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for ssel usage
    fn as_ssel(&self);
}

/// SPI driver.
pub struct Spi<'d, T: Instance, M: Mode> {
    inner: PeripheralRef<'d, T>,
    phantom: PhantomData<(&'d mut T, M)>,
}

fn calc_prescs(freq: u32) -> (u8, u8) {
    todo!();
}

impl<'d, FC: Instance, M: Mode> Spi<'d, FC, M> {
    fn new_inner(
        inner: impl Peripheral<P = FC> + 'd,
        sclk: impl SckPin<FC> + 'd,
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
        sclk.as_sclk();
        mosi.as_mosi();
        miso.as_miso();
        ssel0.as_ssel();
        ssel1.as_ssel();
        ssel2.as_ssel();
        ssel3.as_ssel();

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

// flexcomm <-> Pin function map
macro_rules! impl_miso {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl MosiPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_mosi(&self) {
                // UM11147 table 299 pg 262+
                self.set_pull(crate::gpio::Pull::None)
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
            fn as_miso(&self) {
                // UM11147 table 299 pg 262+
                self.set_pull(crate::gpio::Pull::None)
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
            fn as_sck(&self) {
                // UM11147 table 299 pg 262+
                self.set_pull(crate::gpio::Pull::None)
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
            fn as_ssel(&self) {
                // UM11147 table 299 pg 262+
                self.set_pull(crate::gpio::Pull::None)
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

// note that signals and pins may be optionally mapped to multiple locations

/// Flexcomm0 SPI GPIO options -
impl_miso!(PIO0_1, F1, FLEXCOMM0);
impl_mosi!(PIO0_2, F1, FLEXCOMM0);
impl_sck!(PIO0_0, F1, FLEXCOMM0);
impl_ssel!(PIO0_3, F1, FLEXCOMM0); // SSEL0
impl_ssel!(PIO0_4, F1, FLEXCOMM0); // SSEL1
impl_ssel!(PIO0_5, F1, FLEXCOMM0); // SSEL2
impl_ssel!(PIO0_6, F1, FLEXCOMM0); // SSEL3
impl_ssel!(PIO0_10, F5, FLEXCOMM0); // SSEL2
impl_ssel!(PIO0_11, F5, FLEXCOMM0); // SSEL3

impl_miso!(PIO3_0, F5, FLEXCOMM0);
impl_mosi!(PIO3_1, F5, FLEXCOMM0);
impl_sck!(PIO3_2, F5, FLEXCOMM0);
impl_ssel!(PIO3_3, F5, FLEXCOMM0); // SSEL0
impl_ssel!(PIO3_4, F5, FLEXCOMM0); // SSEL1
impl_ssel!(PIO3_5, F5, FLEXCOMM0); // SSEL2
impl_ssel!(PIO3_6, F5, FLEXCOMM0); // SSEL3

/// Flexcomm1 SPI GPIO options -
impl_ssel!(PIO0_3, F5, FLEXCOMM1); // SSEL2
impl_ssel!(PIO0_4, F5, FLEXCOMM1); // SSEL3
impl_sck!(PIO0_7, F1, FLEXCOMM1);
impl_miso!(PIO0_8, F1, FLEXCOMM1);
impl_mosi!(PIO0_9, F1, FLEXCOMM1);
impl_ssel!(PIO0_10, F1, FLEXCOMM1); // SSEL0
impl_ssel!(PIO0_11, F1, FLEXCOMM1); // SSEL1
impl_ssel!(PIO0_12, F1, FLEXCOMM1); // SSEL2
impl_ssel!(PIO0_13, F1, FLEXCOMM1); // SSEL3

impl_sck!(PIO7_25, F1, FLEXCOMM1);
impl_miso!(PIO7_26, F1, FLEXCOMM1);
impl_mosi!(PIO7_27, F1, FLEXCOMM1);
impl_ssel!(PIO7_28, F1, FLEXCOMM1); // SSEL0
impl_ssel!(PIO7_29, F1, FLEXCOMM1); // SSEL1
impl_ssel!(PIO7_30, F1, FLEXCOMM1); // SSEL2
impl_ssel!(PIO7_31, F1, FLEXCOMM1); // SSEL3

/// Flexcomm2 SPI GPIO options -
impl_sck!(PIO0_14, F1, FLEXCOMM2);
impl_miso!(PIO0_15, F1, FLEXCOMM2);
impl_mosi!(PIO0_16, F1, FLEXCOMM2);
impl_ssel!(PIO0_17, F1, FLEXCOMM2); // SSEL0
impl_ssel!(PIO0_18, F1, FLEXCOMM2); // SSEL1
impl_ssel!(PIO0_19, F1, FLEXCOMM2); // SSEL2
impl_ssel!(PIO0_20, F1, FLEXCOMM2); // SSEL3
impl_ssel!(PIO0_24, F5, FLEXCOMM2); // SSEL2
impl_ssel!(PIO0_25, F5, FLEXCOMM2); // SSEL3

impl_ssel!(PIO4_8, F5, FLEXCOMM2); // SSEL2

impl_sck!(PIO7_24, F5, FLEXCOMM2);
impl_miso!(PIO7_30, F5, FLEXCOMM2);
impl_mosi!(PIO7_31, F5, FLEXCOMM2);

/// Flexcomm3 SPI GPIO options -
// todo for other fcn channels...

/// Flexcomm4 SPI GPIO options -

/// Flexcomm5 SPI GPIO options -

/// Flexcomm6 SPI GPIO options -

/// Flexcomm7 SPI GPIO options -

/// Flexcomm14 SPI GPIO options -

/// Flexcomm15 SPI GPIO options -

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
