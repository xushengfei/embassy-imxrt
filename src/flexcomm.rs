//!FLEXCOMM
//!
#![macro_use]

use core::ptr;

use crate::peripherals;
use crate::peripherals::FLEXCOMM0;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::pac::flexcomm0;
use mimxrt685s_pac as pac;

// Re-export SVD variants to allow user to directly set values.
pub use pac::clkctl1::flexcomm::fcfclksel::Sel as FcClksel;
pub use pac::flexcomm0::pselid::Lock as FlexcommLock;
pub use pac::flexcomm0::pselid::Persel as Function;

/// Flexcomm error types
#[non_exhaustive]
pub enum ConfigError {
    /// general purpose error
    InvalidConfig,
}

/// Flexcomm Config structure, containing:
/// function: SPI, UART, I2C from svd
/// lock: whether or not to lock the pselid
/// non-exhaustive because future upgrades may add config item
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    pub function: Function, // serial comm peripheral type
    pub lock: FlexcommLock, // lock the FC, or not
    pub clksel: FcClksel,   // required clock source
}

// a safe default for peripheral drivers to pre-init their configs
impl Default for Config {
    fn default() -> Self {
        Config {
            function: Function::NoPeriphSelected,
            lock: FlexcommLock::Unlocked,
            clksel: FcClksel::None,
        }
    }
}

/// Generic flexcomm struct
/// Flexcomm n channels (0-7, 14,15)
pub struct Flexcomm<'d, T: FlexcommInstance> {
    _fc: PeripheralRef<'d, T>,
    config: Config,
}

/// Flexcomm channel-specific implementations
impl<'d, T: FlexcommInstance> Flexcomm<'d, T> {
    // This is constrained to only accepting a type that implements
    // the peripheral trait, which is further constrained by only
    // implementing our FlexcommInstance trait.
    pub fn new(instance: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        // Converts the passed in peripheral to a peripheral reference
        into_ref!(instance);

        // Create our struct
        let fc = Self { _fc: instance, config };

        // TODO: check if fc channel is already configured and/or locked?

        fc
    }

    // flexcomm owns the associated external config and control bits in SYSCON and RST_CTL.
    // TODO: peripheral drivers must tell flexcomm which source clock to select (add it to Config struct).

    /// enable channel and connect source clock
    /// Need config information: Function, Lock, and source clock to use
    pub fn enable(&self) {
        // Enable the Flexcomm channel
        T::select_clock(self.config.clksel);
        T::enable_clock();
        T::reset_peripheral();
        self.set_function_and_lock();
    }

    /// disable channel and disconnect associated source clock
    pub fn disable(&self) {
        // Disable the Flexcomm channel
        T::disable_clock();
        T::deselect_clock();
    }

    /// Determine clock freq of actual source clock
    fn calculate_clock_frequency(&self) -> u32 {
        let mut freqHz: u32 = 0;
        // TODO: determine source clock frequency configured for this fc
        freqHz
    }

    /// Set the peripheral function and optionally lock
    fn set_function_and_lock(&self) {
        // TODO: Check if peripheral is present, return error if not?
        // TODO: Check if peripheral is locked or mapped to a diff peripheral?

        match self.config.function {
            Function::NoPeriphSelected => {
                // Set the peripheral function to No peripheral selected
                T::fc_reg().pselid().write(|w| w.persel().no_periph_selected());
            }
            Function::Usart => {
                // Set the peripheral function to USART
                if T::fc_reg().pselid().read().usartpresent().is_present() {
                    T::fc_reg().pselid().write(|w| w.persel().usart());
                }
            }
            Function::Spi => {
                // Set the peripheral function to SPI
                if T::fc_reg().pselid().read().spipresent().is_present() {
                    T::fc_reg().pselid().write(|w| w.persel().spi());
                }
            }
            Function::I2c => {
                // Set the peripheral function to I2C
                if T::fc_reg().pselid().read().i2cpresent().is_present() {
                    T::fc_reg().pselid().write(|w| w.persel().i2c());
                }
            }
            Function::I2sReceive => {
                // Set the peripheral function to I2S
                if T::fc_reg().pselid().read().i2spresent().is_present() {
                    T::fc_reg().pselid().write(|w| w.persel().i2s_receive());
                }
            }
            Function::I2sTransmit => {
                // Set the peripheral function to I2S
                if T::fc_reg().pselid().read().i2spresent().is_present() {
                    T::fc_reg().pselid().write(|w| w.persel().i2s_transmit());
                }
            }
        }

        if self.config.lock == FlexcommLock::Locked {
            T::fc_reg().pselid().modify(|_, w| w.lock().locked());
        } else {
            T::fc_reg().pselid().modify(|_, w| w.lock().unlocked());
        }
    }
}

// Sealed to prevent it from being implemented
// on any arbitrary type outside this module.
trait SealedFlexcommInstance {
    // All flexcomm registerblocks are derived from flexcomm0.
    // They all have the same properties, except fc14 is SPI only and fc15 is I2C only
    fn fc_reg() -> &'static crate::pac::flexcomm0::RegisterBlock;

    // reset the fc channel
    fn reset_peripheral();

    /// select associated source clock (SYSCON CLKCTL1_FCnFCLKSEL)
    //fn select_clock(&self) {}
    fn select_clock(clksel: FcClksel) {}

    /// deselect associated source clock (SYSCON CLKCTL1_FCnFCLKSEL)
    //fn deselect_clock(&self) {}
    fn deselect_clock() {}

    /// Enable the source clock (SYSCON CLKCTL1_PSCCTL0)
    fn enable_clock();

    /// Disable the source clock (SYSCON CLKCTL1_PSCCTL0)
    fn disable_clock();
}

#[allow(private_bounds)]
pub trait FlexcommInstance: SealedFlexcommInstance {}

// macro to replicate FlexcommInstance traits for all fc channel register sets

macro_rules! impl_instance {
    ($fc_periph:ident, $fc_reg_block:ident, $fcn_clk_set:ident, $fcn_clk_clr:ident, $fcn_rst_set:ident, $fcn_rst_clr:ident, $fcn_rst:ident, $fcn_sel:literal) => {
        // Implement the actual private trait
        impl SealedFlexcommInstance for crate::peripherals::$fc_periph {
            fn fc_reg() -> &'static crate::pac::flexcomm0::RegisterBlock {
                // This grabs the pointer to the specific flexcomm peripheral
                // SAFETY: safe if executed from single executor context or during initialization only
                unsafe { &*crate::pac::$fc_reg_block::ptr() }
            }

            /// select associated source clock (SYSCON CLKCTL1_FC1FCLKSEL)
            fn select_clock(clksel: FcClksel) {
                // fc 0 - 7 addressed with flexcomm(n).fcfclksel()
                // fc 14 addressed with .fc14fclksel()
                // fc 15 addressed with .fc15fclksel()

                // SAFETY: safe from single executor
                let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };

                if ($fcn_sel >= 0) && ($fcn_sel <= 7) {
                    match clksel {
                        FcClksel::SfroClk => clkctl1
                            .flexcomm($fcn_sel)
                            .fcfclksel()
                            .write(|w| w.sel().sfro_clk()),
                        FcClksel::FfroClk => clkctl1
                            .flexcomm($fcn_sel)
                            .fcfclksel()
                            .write(|w| w.sel().ffro_clk()),
                        FcClksel::AudioPllClk => clkctl1
                            .flexcomm($fcn_sel)
                            .fcfclksel()
                            .write(|w| w.sel().audio_pll_clk()),
                        FcClksel::MasterClk => clkctl1
                            .flexcomm($fcn_sel)
                            .fcfclksel()
                            .write(|w| w.sel().master_clk()),
                        FcClksel::FcnFrgClk => clkctl1
                            .flexcomm($fcn_sel)
                            .fcfclksel()
                            .write(|w| w.sel().fcn_frg_clk()),
                        FcClksel::None => clkctl1.flexcomm($fcn_sel).fcfclksel().write(|w| w.sel().none()),
                    }
                } else if $fcn_sel == 14 {
                    let fc14clksel = clkctl1.fc14fclksel();
                    match clksel {
                        FcClksel::SfroClk => fc14clksel.write(|w| w.sel().sfro_clk()),
                        FcClksel::FfroClk => fc14clksel.write(|w| w.sel().ffro_clk()),
                        FcClksel::AudioPllClk => fc14clksel.write(|w| w.sel().audio_pll_clk()),
                        FcClksel::MasterClk => fc14clksel.write(|w| w.sel().master_clk()),
                        FcClksel::FcnFrgClk => fc14clksel.write(|w| w.sel().fcn_frg_clk()),
                        FcClksel::None => fc14clksel.write(|w| w.sel().none()),
                    }
                } else if $fcn_sel == 15 {
                    let fc15clksel = clkctl1.fc15fclksel();
                    match clksel {
                        FcClksel::SfroClk => fc15clksel.write(|w| w.sel().sfro_clk()),
                        FcClksel::FfroClk => fc15clksel.write(|w| w.sel().ffro_clk()),
                        FcClksel::AudioPllClk => fc15clksel.write(|w| w.sel().audio_pll_clk()),
                        FcClksel::MasterClk => fc15clksel.write(|w| w.sel().master_clk()),
                        FcClksel::FcnFrgClk => fc15clksel.write(|w| w.sel().fcn_frg_clk()),
                        FcClksel::None => fc15clksel.write(|w| w.sel().none()),
                    }
                } else {
                    panic!();
                }
            }

            /// deselect associated source clock (SYSCON CLKCTL1_FC1FCLKSEL)
            fn deselect_clock() {
                let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };
                let mut fcfclksel = clkctl1.flexcomm(0).fcfclksel(); //default

                // fc 0 - 7 addressed with flexcomm(n).fcfclksel()
                // fc 14 addressed with .fc14fclksel()
                // fc 15 addressed with .fc15fclksel()

                match $fcn_sel {
                    0 => {
                        fcfclksel = clkctl1.flexcomm(0).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    1 => {
                        fcfclksel = clkctl1.flexcomm(1).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    2 => {
                        fcfclksel = clkctl1.flexcomm(2).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    3 => {
                        fcfclksel = clkctl1.flexcomm(3).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    4 => {
                        fcfclksel = clkctl1.flexcomm(4).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    5 => {
                        fcfclksel = clkctl1.flexcomm(5).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    6 => {
                        fcfclksel = clkctl1.flexcomm(6).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    7 => {
                        fcfclksel = clkctl1.flexcomm(7).fcfclksel();
                        fcfclksel.write(|w| w.sel().none());
                    }
                    14 => {
                        let fc14clksel = clkctl1.fc14fclksel();
                        fc14clksel.write(|w| w.sel().none());
                    }
                    15 => {
                        let fc15clksel = clkctl1.fc15fclksel();
                        fc15clksel.write(|w| w.sel().none());
                    }
                    _ => {
                        panic!();
                    }
                }
            }

            fn enable_clock() {
                // SAFETY: safe if executed from single executor context or during initialization only. Write to "Set" register affects only the specific bit being touched
                let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };
                clkctl1.pscctl0_set().write(|w| w.$fcn_clk_set().set_bit());
            }

            fn disable_clock() {
                // SAFETY: safe if executed from single executor context or during initialization only. Write to "Clr" register affects only the specific bit being touched
                let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };
                clkctl1.pscctl0_clr().write(|w| w.$fcn_clk_clr().set_bit());
            }

            fn reset_peripheral() {
                // SAFETY: safe if executed from single executor context or during initialization only. Write to "Set" and "Clr" registers affects only the specific bit being touched
                let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };

                // set reset
                rstctl1.prstctl0_set().write(|w| w.$fcn_rst_set().set_reset());
                while rstctl1.prstctl0().read().$fcn_rst().bit_is_clear() {}

                // clear reset
                rstctl1.prstctl0_clr().write(|w| w.$fcn_rst_clr().clr_reset());
                while rstctl1.prstctl0().read().$fcn_rst().bit_is_set() {}
            }
        }

        impl FlexcommInstance for crate::peripherals::$fc_periph {}
    };
}

// Implement the FlexcommInstance traits for every flexcomm peripheral
impl_instance!(
    FLEXCOMM0,
    Flexcomm0,
    fc0_clk_set,
    fc0_clk_clr,
    flexcomm0_rst_set,
    flexcomm0_rst_clr,
    flexcomm0_rst,
    0
);

impl_instance!(
    FLEXCOMM1,
    Flexcomm1,
    fc1_clk_set,
    fc1_clk_clr,
    flexcomm1_rst_set,
    flexcomm1_rst_clr,
    flexcomm1_rst,
    1
);

impl_instance!(
    FLEXCOMM2,
    Flexcomm2,
    fc2_clk_set,
    fc2_clk_clr,
    flexcomm2_rst_set,
    flexcomm2_rst_clr,
    flexcomm2_rst,
    2
);

impl_instance!(
    FLEXCOMM3,
    Flexcomm3,
    fc3_clk_set,
    fc3_clk_clr,
    flexcomm3_rst_set,
    flexcomm3_rst_clr,
    flexcomm3_rst,
    3
);

impl_instance!(
    FLEXCOMM4,
    Flexcomm4,
    fc4_clk_set,
    fc4_clk_clr,
    flexcomm4_rst_set,
    flexcomm4_rst_clr,
    flexcomm4_rst,
    4
);

impl_instance!(
    FLEXCOMM5,
    Flexcomm5,
    fc5_clk_set,
    fc5_clk_clr,
    flexcomm5_rst_set,
    flexcomm5_rst_clr,
    flexcomm5_rst,
    5
);

impl_instance!(
    FLEXCOMM6,
    Flexcomm6,
    fc6_clk_set,
    fc6_clk_clr,
    flexcomm6_rst_set,
    flexcomm6_rst_clr,
    flexcomm6_rst,
    6
);

impl_instance!(
    FLEXCOMM7,
    Flexcomm7,
    fc7_clk_set,
    fc7_clk_clr,
    flexcomm7_rst_set,
    flexcomm7_rst_clr,
    flexcomm7_rst,
    7
);

// 14 is SPI only
impl_instance!(
    FLEXCOMM14,
    Flexcomm14,
    fc14_spi_clk_set,
    fc14_spi_clk_clr,
    flexcomm14_spi_rst_set,
    flexcomm14_spi_rst_clr,
    flexcomm14_spi_rst,
    14
);

// 15 is I2C only
impl_instance!(
    FLEXCOMM15,
    Flexcomm15,
    fc15_i2c_clk_set,
    fc15_i2c_clk_clr,
    flexcomm15_i2c_rst_set,
    flexcomm15_i2c_rst_clr,
    flexcomm15_i2c_rst,
    15
);
