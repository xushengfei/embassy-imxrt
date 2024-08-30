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
                            // TBD: Specify preferred source clock? ex: low speed / high speed / pll / external
}

// a safe default for peripheral drivers to pre-init their configs
impl Default for Config {
    fn default() -> Self {
        Config {
            function: Function::NoPeriphSelected,
            lock: FlexcommLock::Unlocked,
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

    // TODO: Use new wip clock traits for all methods
    // TBD: Does flexcomm own the associated external config and control bits in SYSCON and RST_CTL ?
    //      If flexcomm does own the external config and control bits, then peripheral drivers
    //      must tell flexcomm which source clock to select (add it to Config struct).

    /// enable channel and connect source clock
    /// Need config information: Function, Lock, and source clock to use
    fn enable(&self) {
        // Enable the Flexcomm channel
        self.attach_clock();
        self.enable_clock();
        self.reset_peripheral();
        self.set_function_and_lock();
    }

    /// disable channel and disconnect associated source clock
    fn disable(&self) {
        // Disable the Flexcomm channel
        self.disable_clock();
        self.detach_clock();
    }

    /// attach associated source clock (SYSCON CLKCTL1_FC1FCLKSEL)
    fn attach_clock(&self) {
        todo!();
    }

    /// detach associated source clock (SYSCON CLKCTL1_FC1FCLKSEL)
    fn detach_clock(&self) {
        todo!();
    }

    /// Enable the source clock (SYSCON CLKCTL1_PSCCTL0)
    fn enable_clock(&self) {
        todo!();
    }

    /// Disable the source clock (SYSCON CLKCTL1_PSCCTL0)
    fn disable_clock(&self) {
        todo!();
    }

    /// Determine clock freq of actual source clock
    fn calculate_clock_frequency(&self) -> u32 {
        let mut freqHz: u32 = 0;
        // TODO: determine source clock frequency configured for this fc
        freqHz
    }

    /// Reset the flexcomm channel RST_CTLn_PSCCTLn
    fn reset_peripheral(&self) {
        // TODO: Reset the correct fc channel
        /* if self == Flexcomm0 */
        {
            // SAFETY: safe so long as executed from single executor context or during initialization only
            let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };

            // set reset
            rstctl1.prstctl0_set().write(|w| w.flexcomm0_rst_set().set_reset());
            while rstctl1.prstctl0().read().flexcomm0_rst().bit_is_clear() {}

            // clear reset
            rstctl1.prstctl0_clr().write(|w| w.flexcomm0_rst_clr().clr_reset());
            while rstctl1.prstctl0().read().flexcomm0_rst().bit_is_set() {}
            todo!();
        }
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
        // TBD: Do we need to support the lock feature?
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
}

#[allow(private_bounds)]
pub trait FlexcommInstance: SealedFlexcommInstance {}

// macro to replicate for multiple FlexcommInstance traits
macro_rules! impl_instance {
    ($fc_periph:ident, $fc_reg_block:ident) => {
        // Implement the actual private trait
        impl SealedFlexcommInstance for crate::peripherals::$fc_periph {
            fn fc_reg() -> &'static crate::pac::flexcomm0::RegisterBlock {
                // This grabs the pointer to the specific flexcomm peripheral
                // SAFETY: safe so long as executed from single executor context or during initialization only
                unsafe { &*crate::pac::$fc_reg_block::ptr() }
            }
        }

        impl FlexcommInstance for crate::peripherals::$fc_periph {}
    };
}

// Implement the FlexcommInstance traits for every flexcomm peripheral
impl_instance!(FLEXCOMM0, Flexcomm0);
impl_instance!(FLEXCOMM1, Flexcomm1);
impl_instance!(FLEXCOMM2, Flexcomm2);
impl_instance!(FLEXCOMM3, Flexcomm3);
impl_instance!(FLEXCOMM4, Flexcomm4);
impl_instance!(FLEXCOMM5, Flexcomm5);
impl_instance!(FLEXCOMM6, Flexcomm6);
impl_instance!(FLEXCOMM7, Flexcomm7);
impl_instance!(FLEXCOMM14, Flexcomm14); // 14 is SPI only
impl_instance!(FLEXCOMM15, Flexcomm15); // 15 is I2C only
