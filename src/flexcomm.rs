//! implements flexcomm interface wrapper for easier usage across modules

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::{interrupt, pac, Peripheral, PeripheralRef};

/// alias for fc0::Registers, as layout is the same across all FCn
pub type FlexcommRegisters = pac::flexcomm0::RegisterBlock;

/// alias for i2c0::Registers, as layout is the same across all FCn
pub type I2cRegisters = pac::i2c0::RegisterBlock;

/// alias for spi0::Registers, as layout is the same across all FCn
pub type SpiRegisters = pac::spi0::RegisterBlock;

/// alias for i2s0::Registers, as layout is the same across all FCn
pub type I2sRegisters = pac::i2s0::RegisterBlock;

/// alias for usart0::Registers, as layout is the same across all FCn
pub type UsartRegisters = pac::usart0::RegisterBlock;

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

    /// AUDIO_PLL
    AudioPll,

    /// MASTER
    Master,

    /// FCn_FRG
    FcnFrg,

    /// disabled
    None,
}

/// what mode to configure this FCn to
pub enum Mode {
    /// i2c operation
    I2c,

    /// spi operation
    Spi,

    /// i2s transmit operation
    I2sTx,

    /// i2s receive operation
    I2sRx,

    /// usart operation
    Usart,

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
    /// feature not present on FCn (such as no SPI or I2S support)
    FeatureNotPresent,
}

/// shorthand for ->Result<T>
pub type Result<T> = core::result::Result<T, Error>;

/// primary low-level flexcomm interface
trait FlexcommLowLevel: sealed::Sealed + Peripheral {
    // fetch the flexcomm register block for direct manipulation
    fn reg() -> &'static FlexcommRegisters;

    // fetch the i2c peripheral registers for this FCn, if they exist
    fn i2c() -> &'static I2cRegisters;

    // fetch the SPI peripheral registers for this FCn, if they exist
    fn spi() -> &'static SpiRegisters;

    // fetch the I2S peripheral registers for this FCn, if they exist
    fn i2s() -> &'static I2sRegisters;

    // fetch the USART peripheral registers for this FCn, if they exist
    fn usart() -> &'static UsartRegisters;

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

/// internal shared SPI peripheral operations
#[allow(private_bounds)]
pub(crate) trait SpiPeripheral: FlexcommLowLevel {}

/// Flexcomm configured for SPI usage
#[allow(private_bounds)]
pub struct SpiBus<'p, F: SpiPeripheral> {
    _fc: PeripheralRef<'p, F>,
}
#[allow(private_bounds)]
impl<'p, F: SpiPeripheral> SpiBus<'p, F> {
    /// use Flexcomm fc as an SPI Bus
    pub fn new(fc: impl SpiPeripheral<P = F> + 'p, clk: Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(Mode::Spi)?;
        Ok(Self { _fc: fc.into_ref() })
    }

    /// retrieve active bus registers
    pub fn spi(&self) -> &'static SpiRegisters {
        F::spi()
    }
}

/// internal shared I2S peripheral operations
#[allow(private_bounds)]
pub(crate) trait I2sPeripheral: FlexcommLowLevel {}

/// Flexcomm configured for I2sTx usage
#[allow(private_bounds)]
pub struct I2sTransmit<'p, F: I2sPeripheral> {
    _fc: PeripheralRef<'p, F>,
}
#[allow(private_bounds)]
impl<'p, F: I2sPeripheral> I2sTransmit<'p, F> {
    /// use Flexcomm fc as an I2sTx Bus
    pub fn new(fc: impl I2sPeripheral<P = F> + 'p, clk: Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(Mode::I2sTx)?;
        Ok(Self { _fc: fc.into_ref() })
    }

    /// retrieve active bus registers
    pub fn i2s(&self) -> &'static I2sRegisters {
        F::i2s()
    }
}

/// Flexcomm configured for I2sRx usage
#[allow(private_bounds)]
pub struct I2sReceive<'p, F: I2sPeripheral> {
    _fc: PeripheralRef<'p, F>,
}
#[allow(private_bounds)]
impl<'p, F: I2sPeripheral> I2sReceive<'p, F> {
    /// use Flexcomm fc as an I2sRx Bus
    pub fn new(fc: impl I2sPeripheral<P = F> + 'p, clk: Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(Mode::I2sRx)?;
        Ok(Self { _fc: fc.into_ref() })
    }

    /// retrieve active bus registers
    pub fn i2s(&self) -> &'static I2sRegisters {
        F::i2s()
    }
}

/// internal shared USARt peripheral operations
#[allow(private_bounds)]
pub(crate) trait UsartPeripheral: FlexcommLowLevel {}

/// Flexcomm configured for USART usage
#[allow(private_bounds)]
pub struct UsartBus<'p, F: UsartPeripheral> {
    _fc: PeripheralRef<'p, F>,
}
#[allow(private_bounds)]
impl<'p, F: UsartPeripheral> UsartBus<'p, F> {
    /// use Flexcomm fc as an USART Bus
    pub fn new(fc: impl UsartPeripheral<P = F> + 'p, clk: Clock) -> Result<Self> {
        F::enable(clk);
        F::set_mode(Mode::Usart)?;
        Ok(Self { _fc: fc.into_ref() })
    }

    /// retrieve active bus registers
    pub fn usart(&self) -> &'static UsartRegisters {
        F::usart()
    }
}

macro_rules! impl_flexcomm {
    ($fcn:expr, $ufc:ident, $lfc:ident, $i2c:ident, $spi:ident, $i2s:ident, $usart:ident, $fc_clk_set:ident, $fc_rst_clr:ident) => {
        impl sealed::Sealed for crate::peripherals::$ufc {}
        impl I2cPeripheral for crate::peripherals::$ufc {}
        impl SpiPeripheral for crate::peripherals::$ufc {}
        impl I2sPeripheral for crate::peripherals::$ufc {}
        impl UsartPeripheral for crate::peripherals::$ufc {}

        impl FlexcommLowLevel for crate::peripherals::$ufc {
            fn reg() -> &'static FlexcommRegisters {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$lfc::ptr() }
            }

            fn i2c() -> &'static I2cRegisters {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$i2c::ptr() }
            }

            fn spi() -> &'static SpiRegisters {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$spi::ptr() }
            }

            fn i2s() -> &'static I2sRegisters {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$i2s::ptr() }
            }

            fn usart() ->&'static UsartRegisters {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$usart::ptr() }
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
                    Mode::Spi => {
                        if fc.pselid().read().spipresent().is_present() {
                            fc.pselid().write(|w| w.persel().spi());
                            Ok(())
                        } else {
                            Err(Error::FeatureNotPresent)
                        }
                    }
                    Mode::I2sTx => {
                        if fc.pselid().read().i2spresent().is_present() {
                            fc.pselid().write(|w| w.persel().i2s_transmit());
                            Ok(())
                        } else {
                            Err(Error::FeatureNotPresent)
                        }
                    }
                    Mode::I2sRx => {
                        if fc.pselid().read().i2spresent().is_present() {
                            fc.pselid().write(|w| w.persel().i2s_receive());
                            Ok(())
                        } else {
                            Err(Error::FeatureNotPresent)
                        }
                    }
                    Mode::Usart => {
                        if fc.pselid().read().usartpresent().is_present() {
                            fc.pselid().write(|w| w.persel().usart());
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

            waker.wake();
        }


    };
}

impl_flexcomm!(
    0,
    FLEXCOMM0,
    Flexcomm0,
    I2c0,
    Spi0,
    I2s0,
    Usart0,
    fc0_clk_set,
    flexcomm0_rst_clr
);
impl_flexcomm!(
    1,
    FLEXCOMM1,
    Flexcomm1,
    I2c1,
    Spi1,
    I2s1,
    Usart1,
    fc1_clk_set,
    flexcomm1_rst_clr
);
impl_flexcomm!(
    2,
    FLEXCOMM2,
    Flexcomm2,
    I2c2,
    Spi2,
    I2s2,
    Usart2,
    fc2_clk_set,
    flexcomm2_rst_clr
);
impl_flexcomm!(
    3,
    FLEXCOMM3,
    Flexcomm3,
    I2c3,
    Spi3,
    I2s3,
    Usart3,
    fc3_clk_set,
    flexcomm3_rst_clr
);
impl_flexcomm!(
    4,
    FLEXCOMM4,
    Flexcomm4,
    I2c4,
    Spi4,
    I2s4,
    Usart4,
    fc4_clk_set,
    flexcomm4_rst_clr
);
impl_flexcomm!(
    5,
    FLEXCOMM5,
    Flexcomm5,
    I2c5,
    Spi5,
    I2s5,
    Usart5,
    fc5_clk_set,
    flexcomm5_rst_clr
);
impl_flexcomm!(
    6,
    FLEXCOMM6,
    Flexcomm6,
    I2c6,
    Spi6,
    I2s6,
    Usart6,
    fc6_clk_set,
    flexcomm6_rst_clr
);
impl_flexcomm!(
    7,
    FLEXCOMM7,
    Flexcomm7,
    I2c7,
    Spi7,
    I2s7,
    Usart7,
    fc7_clk_set,
    flexcomm7_rst_clr
);

// TODO: in follow up flexcomm PR, implement special FC14 + FC15 support
//impl_flexcomm!(14, FLEXCOMM14, Flexcomm14, I2c14, Spi14, I2s14, Usart14);
//impl_flexcomm!(15, FLEXCOMM15, Flexcomm15, I2c15, Sp157, I2s15, Usart15);
