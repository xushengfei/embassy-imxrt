//! implements flexcomm interface wrapper for easier usage across modules

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use paste::paste;

use crate::{interrupt, pac, Peripheral, PeripheralRef};

/// alias for `fc0::Registers`, as layout is the same across all `FCn`
pub type FlexcommRegisters = pac::flexcomm0::RegisterBlock;

/// alias for `i2c0::Registers`, as layout is the same across all `FCn`
pub type I2cRegisters = pac::i2c0::RegisterBlock;

const FC_COUNT: usize = 8;
// One waker per FC
static FC_WAKERS: [AtomicWaker; FC_COUNT] = [const { AtomicWaker::new() }; FC_COUNT];

/// clock selection option
#[derive(Copy, Clone, Debug)]
pub enum Clock {
    /// SFRO
    Sfro,

    /// FFRO
    Ffro,

    /// `AUDIO_PLL`
    AudioPll,

    /// MASTER
    Master,

    /// `FCn_FRG`
    FcnFrg,

    /// disabled
    None,
}

/// what mode to configure this `FCn` to
pub enum Mode {
    /// i2c operation
    I2c,

    /// no peripheral function selected
    None,
}

/// do not allow implementation of trait outside this mod
mod sealed {
    /// trait does not get re-exported outside flexcomm mod, allowing us to safely expose only desired APIs
    pub trait Sealed {}
}

/// potential configuration error reporting
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// feature not present on `FCn` (such as no SPI or I2S support)
    FeatureNotPresent,
}

/// shorthand for ->Result<T>
pub type Result<T> = core::result::Result<T, Error>;

/// primary low-level flexcomm interface
pub(crate) trait FlexcommLowLevel: sealed::Sealed + Peripheral {
    // fetch the flexcomm register block for direct manipulation
    fn reg() -> &'static FlexcommRegisters;

    // fetch the i2c peripheral registers for this FCn, if they exist
    fn i2c() -> &'static I2cRegisters;

    // set the clock select for this flexcomm instance and remove from reset
    fn enable(clk: Clock);

    // attempt to configure bus to operating mode
    fn set_mode(mode: Mode) -> Result<()>;

    // enable interrupt
    unsafe fn enable_interrupt();

    // fetch waker
    fn waker() -> &'static AtomicWaker;
}

/// internal shared I2C peripheral operations
#[allow(private_bounds)]
pub(crate) trait I2cPeripheral: FlexcommLowLevel {}

/// Flexcomm configured for I2C usage
#[allow(private_bounds)]
pub struct I2cBus<'p, F: I2cPeripheral> {
    _fc: PeripheralRef<'p, F>,
}
#[allow(private_bounds)]
impl<'p, F: I2cPeripheral> I2cBus<'p, F> {
    /// use Flexcomm fc as a blocking I2c Bus
    pub fn new_blocking(fc: impl Peripheral<P = F> + 'p, clk: Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(Mode::I2c)?;
        Ok(Self { _fc: fc.into_ref() })
    }

    /// use Flexcomm fc as an async I2c Bus
    pub fn new_async(fc: impl Peripheral<P = F> + 'p, clk: Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(Mode::I2c)?;
        // SAFETY: flexcomm interrupt should be managed through this
        //         interface only
        unsafe { F::enable_interrupt() };
        Ok(Self { _fc: fc.into_ref() })
    }

    /// retrieve active bus registers
    pub fn i2c(&self) -> &'static I2cRegisters {
        F::i2c()
    }

    /// return a waker
    pub fn waker(&self) -> &'static AtomicWaker {
        F::waker()
    }
}

macro_rules! impl_flexcomm {
    ($fcn:expr, $ufc:ident, $lfc:ident, $i2c:ident, $fc_clk_set:ident, $fc_rst_clr:ident) => {
        impl sealed::Sealed for crate::peripherals::$ufc {}
        impl I2cPeripheral for crate::peripherals::$ufc {}

        impl FlexcommLowLevel for crate::peripherals::$ufc {
            fn reg() -> &'static FlexcommRegisters {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$lfc::ptr() }
            }

            fn i2c() -> &'static I2cRegisters {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$i2c::ptr() }
            }

            fn enable(clk: Clock) {
                // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc[n]_clk

                // SAFETY: safe from single executor
                let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };

                clkctl1.pscctl0_set().write(|w| w.$fc_clk_set().set_clock());

                clkctl1.flexcomm($fcn).fcfclksel().write(|w| match clk {
                    Clock::Sfro => w.sel().sfro_clk(),
                    Clock::Ffro => w.sel().ffro_clk(),
                    Clock::AudioPll => w.sel().audio_pll_clk(),
                    Clock::Master => w.sel().master_clk(),
                    Clock::FcnFrg => w.sel().fcn_frg_clk(),
                    Clock::None => w.sel().none(),
                });
                clkctl1.flexcomm($fcn).frgclksel().write(|w| match clk {
                    Clock::Sfro => w.sel().sfro_clk(),
                    Clock::Ffro => w.sel().ffro_clk(),
                    Clock::AudioPll => w.sel().frg_pll_clk(),
                    Clock::Master => w.sel().main_clk(),
                    Clock::FcnFrg => w.sel().frg_pll_clk(),
                    Clock::None => w.sel().none(),
                });
                clkctl1
                    .flexcomm($fcn)
                    .frgctl()
                    .write(|w|
                        // SAFETY: unsafe only used for .bits() call
                        unsafe { w.mult().bits(0) });

                // SAFETY: safe from single executor
                let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };

                rstctl1.prstctl0_clr().write(|w| w.$fc_rst_clr().set_bit());
            }

            fn set_mode(mode: Mode) -> Result<()> {
                let fc = Self::reg();

                match mode {
                    Mode::I2c => {
                        if fc.pselid().read().i2cpresent().is_present() {
                            fc.pselid().write(|w| w.persel().i2c());
                            Ok(())
                        } else {
                            Err(Error::FeatureNotPresent)
                        }
                    }
                    Mode::None => {
                        fc.pselid().write(|w| w.persel().no_periph_selected());
                        Ok(())
                    }
                }
            }

            unsafe fn enable_interrupt() {
                interrupt::$ufc.unpend();
                interrupt::$ufc.enable();
            }

            fn waker() -> &'static AtomicWaker {
                &FC_WAKERS[$fcn]
            }
        }

        #[cfg(feature = "rt")]
        #[interrupt]
        #[allow(non_snake_case)]
        fn $ufc() {
            let waker = &FC_WAKERS[$fcn];

            // SAFETY: this will be the only accessor to this flexcomm's
            //         i2c block
            let i2c = unsafe { &*crate::pac::$i2c::ptr() };

            if i2c.intstat().read().mstpending().bit_is_set() {
                i2c.intenclr().write(|w| w.mstpendingclr().set_bit());
            }

            if i2c.intstat().read().mstarbloss().bit_is_set() {
                i2c.intenclr().write(|w| w.mstarblossclr().set_bit());
            }

            if i2c.intstat().read().mstststperr().bit_is_set() {
                i2c.intenclr().write(|w| w.mstststperrclr().set_bit());
            }

            if i2c.intstat().read().slvpending().bit_is_set() {
                i2c.intenclr().write(|w| w.slvpendingclr().set_bit());
            }

            if i2c.intstat().read().slvdesel().bit_is_set() {
                i2c.intenclr().write(|w| w.slvdeselclr().set_bit());
            }

            waker.wake();
        }


    };
}

impl_flexcomm!(0, FLEXCOMM0, Flexcomm0, I2c0, fc0_clk_set, flexcomm0_rst_clr);
impl_flexcomm!(1, FLEXCOMM1, Flexcomm1, I2c1, fc1_clk_set, flexcomm1_rst_clr);
impl_flexcomm!(2, FLEXCOMM2, Flexcomm2, I2c2, fc2_clk_set, flexcomm2_rst_clr);
impl_flexcomm!(3, FLEXCOMM3, Flexcomm3, I2c3, fc3_clk_set, flexcomm3_rst_clr);
impl_flexcomm!(4, FLEXCOMM4, Flexcomm4, I2c4, fc4_clk_set, flexcomm4_rst_clr);
impl_flexcomm!(5, FLEXCOMM5, Flexcomm5, I2c5, fc5_clk_set, flexcomm5_rst_clr);
impl_flexcomm!(6, FLEXCOMM6, Flexcomm6, I2c6, fc6_clk_set, flexcomm6_rst_clr);
impl_flexcomm!(7, FLEXCOMM7, Flexcomm7, I2c7, fc7_clk_set, flexcomm7_rst_clr);

macro_rules! declare_into_mode {
    ($mode:ident) => {
        paste! {
            /// Sealed Mode trait
            trait [<SealedInto $mode:camel>]: FlexcommLowLevel {}

            /// Select mode of operation
            #[allow(private_bounds)]
            pub trait [<Into $mode:camel>]: [<SealedInto $mode:camel>] {
                /// Set mode of operation
                fn [<into_ $mode>]() {
                    Self::reg().pselid().write(|w| w.persel().[<$mode>]());
                }
            }
        }
    };
}

macro_rules! impl_into_mode {
    ($mode:ident, $($fc:ident),*) => {
	$(
	    paste!{
		impl [<SealedInto $mode:camel>] for crate::peripherals::$fc {}
		impl [<Into $mode:camel>] for crate::peripherals::$fc {}
	    }
	)*
    }
}

declare_into_mode!(usart);
impl_into_mode!(usart, FLEXCOMM0, FLEXCOMM1, FLEXCOMM2, FLEXCOMM3, FLEXCOMM4, FLEXCOMM5, FLEXCOMM6, FLEXCOMM7);

// REVISIT: Add support for FLEXCOMM14
declare_into_mode!(spi);
impl_into_mode!(spi, FLEXCOMM0, FLEXCOMM1, FLEXCOMM2, FLEXCOMM3, FLEXCOMM4, FLEXCOMM5, FLEXCOMM6, FLEXCOMM7);

// REVISIT: Add support for FLEXCOMM15
declare_into_mode!(i2c);
impl_into_mode!(i2c, FLEXCOMM0, FLEXCOMM1, FLEXCOMM2, FLEXCOMM3, FLEXCOMM4, FLEXCOMM5, FLEXCOMM6, FLEXCOMM7);

declare_into_mode!(i2s_transmit);
impl_into_mode!(
    i2s_transmit,
    FLEXCOMM0,
    FLEXCOMM1,
    FLEXCOMM2,
    FLEXCOMM3,
    FLEXCOMM4,
    FLEXCOMM5,
    FLEXCOMM6,
    FLEXCOMM7
);

declare_into_mode!(i2s_receive);
impl_into_mode!(
    i2s_receive,
    FLEXCOMM0,
    FLEXCOMM1,
    FLEXCOMM2,
    FLEXCOMM3,
    FLEXCOMM4,
    FLEXCOMM5,
    FLEXCOMM6,
    FLEXCOMM7
);

// TODO: in follow up flexcomm PR, implement special FC14 + FC15 support
//impl_flexcomm!(14, FLEXCOMM14, Flexcomm14, I2c14, Spi14, I2s14);
//impl_flexcomm!(15, FLEXCOMM15, Flexcomm15, I2c15, Sp157, I2s15);
