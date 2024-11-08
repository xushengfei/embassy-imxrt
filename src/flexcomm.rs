//! implements flexcomm interface wrapper for easier usage across modules

use paste::paste;

use crate::{pac, Peripheral};

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

/// do not allow implementation of trait outside this mod
mod sealed {
    /// trait does not get re-exported outside flexcomm mod, allowing us to safely expose only desired APIs
    pub trait Sealed {}
}

/// primary low-level flexcomm interface
pub(crate) trait FlexcommLowLevel: sealed::Sealed + Peripheral {
    // fetch the flexcomm register block for direct manipulation
    fn reg() -> &'static pac::flexcomm0::RegisterBlock;

    // set the clock select for this flexcomm instance and remove from reset
    fn enable(clk: Clock);
}

macro_rules! impl_flexcomm {
    ($fcn:expr, $ufc:ident, $lfc:ident, $fc_clk_set:ident, $fc_rst_clr:ident) => {
        impl sealed::Sealed for crate::peripherals::$ufc {}

        impl FlexcommLowLevel for crate::peripherals::$ufc {
            fn reg() -> &'static pac::flexcomm0::RegisterBlock {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$lfc::ptr() }
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
        }
    };
}

impl_flexcomm!(0, FLEXCOMM0, Flexcomm0, fc0_clk_set, flexcomm0_rst_clr);
impl_flexcomm!(1, FLEXCOMM1, Flexcomm1, fc1_clk_set, flexcomm1_rst_clr);
impl_flexcomm!(2, FLEXCOMM2, Flexcomm2, fc2_clk_set, flexcomm2_rst_clr);
impl_flexcomm!(3, FLEXCOMM3, Flexcomm3, fc3_clk_set, flexcomm3_rst_clr);
impl_flexcomm!(4, FLEXCOMM4, Flexcomm4, fc4_clk_set, flexcomm4_rst_clr);
impl_flexcomm!(5, FLEXCOMM5, Flexcomm5, fc5_clk_set, flexcomm5_rst_clr);
impl_flexcomm!(6, FLEXCOMM6, Flexcomm6, fc6_clk_set, flexcomm6_rst_clr);
impl_flexcomm!(7, FLEXCOMM7, Flexcomm7, fc7_clk_set, flexcomm7_rst_clr);

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
