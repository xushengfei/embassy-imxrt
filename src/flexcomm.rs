//!FLEXCOMM
//!
#![macro_use]

use core::ptr;

use crate::peripherals;
use embassy_hal_internal::{impl_peripheral, into_ref, Peripheral, PeripheralRef};

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
pub struct Config {
    function: Function, // serial comm peripheral type
    lock: FlexcommLock, // lock the FC, or not
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

/// Flexcomm traits
trait Flexcomm {
    /// enable channel and connect source clock
    /// Need config information: Function, Lock, and source clock to use
    fn enable(&mut self) {
        // Enable the Flexcomm connector
        //self.attach_clock();
        //self.enable_clock();
        //self.reset_peripheral();
        //self.calculate_clock_frequency();
        //self.set_function_and_lock();
    }

    /// disable channel and disconnect associated source clock
    fn disable(&self) {
        // Disable the Flexcomm connector
        //self.disable_clock();
        //self.detach_clock();
    }

    /// attach associated source clock (SYSCON CLKCTL1_FC1FCLKSEL)
    fn attach_clock(&self) {}

    /// detach associated source clock (SYSCON CLKCTL1_FC1FCLKSEL)
    fn detach_clock(&self) {}

    /// Enable the source clock (SYSCON CLKCTL1_PSCCTL0)
    fn enable_clock(&self) {}

    /// Disable the source clock (SYSCON CLKCTL1_PSCCTL0)
    fn disable_clock(&self) {}

    /// Determine clock freq of actual source clock
    fn calculate_clock_frequency(&mut self) {}

    /// Reset the flexcomm channel RST_CTLn_PSCCTLn
    fn reset_peripheral(&self) {}

    /// Set the peripheral function and optionally lock
    fn set_function_and_lock(&self) {}
}

/// Flexcomm channels 0-7, 14,15
struct Flexcomm0 {
    config: Config,
}
struct Flexcomm1 {
    config: Config,
}
struct Flexcomm2 {
    config: Config,
}
struct Flexcomm3 {
    config: Config,
}
struct Flexcomm4 {
    config: Config,
}
struct Flexcomm5 {
    config: Config,
}
struct Flexcomm6 {
    config: Config,
}
struct Flexcomm7 {
    config: Config,
}
struct Flexcomm14 {
    config: Config,
}
struct Flexcomm15 {
    config: Config,
}

/// Flexcomm channel-specific implementations
impl Flexcomm0 {
    fn new(config: Config) -> Flexcomm0 {
        Flexcomm0 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm0::RegisterBlock {
        unsafe { &*(pac::Flexcomm0::ptr() as *const pac::flexcomm0::RegisterBlock) }
    }
}

impl Flexcomm1 {
    fn new(config: Config) -> Flexcomm1 {
        Flexcomm1 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm1::RegisterBlock {
        unsafe { &*(pac::Flexcomm1::ptr() as *const pac::flexcomm1::RegisterBlock) }
    }
}

impl Flexcomm2 {
    fn new(config: Config) -> Flexcomm2 {
        Flexcomm2 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm2::RegisterBlock {
        unsafe { &*(pac::Flexcomm2::ptr() as *const pac::flexcomm2::RegisterBlock) }
    }
}

impl Flexcomm3 {
    fn new(config: Config) -> Flexcomm3 {
        Flexcomm3 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm3::RegisterBlock {
        unsafe { &*(pac::Flexcomm3::ptr() as *const pac::flexcomm3::RegisterBlock) }
    }
}

impl Flexcomm4 {
    fn new(config: Config) -> Flexcomm4 {
        Flexcomm4 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm4::RegisterBlock {
        unsafe { &*(pac::Flexcomm4::ptr() as *const pac::flexcomm4::RegisterBlock) }
    }
}

impl Flexcomm5 {
    fn new(config: Config) -> Flexcomm5 {
        Flexcomm5 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm5::RegisterBlock {
        unsafe { &*(pac::Flexcomm5::ptr() as *const pac::flexcomm5::RegisterBlock) }
    }
}

impl Flexcomm6 {
    fn new(config: Config) -> Flexcomm6 {
        Flexcomm6 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm6::RegisterBlock {
        unsafe { &*(pac::Flexcomm6::ptr() as *const pac::flexcomm6::RegisterBlock) }
    }
}

impl Flexcomm7 {
    fn new(config: Config) -> Flexcomm7 {
        Flexcomm7 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm7::RegisterBlock {
        unsafe { &*(pac::Flexcomm7::ptr() as *const pac::flexcomm7::RegisterBlock) }
    }
}

// 14 is SPI only
impl Flexcomm14 {
    fn new(config: Config) -> Flexcomm14 {
        Flexcomm14 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm14::RegisterBlock {
        unsafe { &*(pac::Flexcomm14::ptr() as *const pac::flexcomm14::RegisterBlock) }
    }
}

// 15 is I2C only
impl Flexcomm15 {
    fn new(config: Config) -> Flexcomm15 {
        Flexcomm15 { config }
    }

    fn regs(&self) -> &'static pac::flexcomm15::RegisterBlock {
        unsafe { &*(pac::Flexcomm15::ptr() as *const pac::flexcomm15::RegisterBlock) }
    }
}

/// Flexcomm channel generic trait implementations
impl Flexcomm for Flexcomm0 {
    // TODO: Use new wip clock traits for all methods
    // TBD: Does flexcomm own the associated external config and control bits in SYSCON and RST_CTL ?
    //      If flexcomm does own the external config and control bits, then peripheral drivers
    //      must tell flexcomm which source clock to select (add it to Config struct).

    /// enable channel and connect source clock
    /// Need config information: Function, Lock, and source clock to use
    fn enable(&mut self) {
        // Enable the Flexcomm connector
        //self.attach_clock();
        //self.enable_clock();
        //self.reset_peripheral();
        //self.calculate_clock_frequency();
        //self.set_reg();
        todo!();
    }

    /// disable channel and disconnect associated source clock
    fn disable(&self) {
        // Disable the Flexcomm connector
        //self.disable_clock();
        //self.detach_clock();
        todo!();
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
    fn calculate_clock_frequency(&mut self) {
        todo!();
    }

    /// Reset the flexcomm channel RST_CTLn_PSCCTLn
    fn reset_peripheral(&self) {
        todo!();
    }

    /// Set the peripheral function and optionally lock
    fn set_function_and_lock(&self) {
        // TODO: Check if peripheral is present.
        // TBD: Check if peripheral is locked or mapped to a diff peripheral?

        match self.config.function {
            Function::NoPeriphSelected => {
                // Set the peripheral function to No peripheral selected
                self.regs().pselid().write(|w| w.persel().no_periph_selected());
            }
            Function::Usart => {
                // Set the peripheral function to USART
                self.regs().pselid().write(|w| w.persel().usart());
            }
            Function::Spi => {
                // Set the peripheral function to SPI
                self.regs().pselid().write(|w| w.persel().spi());
            }
            Function::I2c => {
                // Set the peripheral function to I2C
                self.regs().pselid().write(|w| w.persel().i2c());
            }
            Function::I2sReceive => {
                // Set the peripheral function to I2S
                self.regs().pselid().write(|w| w.persel().i2s_receive());
            }
            Function::I2sTransmit => {
                // Set the peripheral function to I2S
                self.regs().pselid().write(|w| w.persel().i2s_transmit());
            }
        }
        // TBD: Do we need to support the lock feature?
        if self.config.lock == FlexcommLock::Locked {
            self.regs().pselid().write(|w| w.lock().locked());
        } else {
            self.regs().pselid().write(|w| w.lock().unlocked());
        }
    }
}

impl Flexcomm for Flexcomm1 {
    // todo
}

impl Flexcomm for Flexcomm2 {
    // todo
}

impl Flexcomm for Flexcomm3 {
    // todo
}

impl Flexcomm for Flexcomm4 {
    // todo
}

impl Flexcomm for Flexcomm5 {
    // todo
}

impl Flexcomm for Flexcomm6 {
    // todo
}

impl Flexcomm for Flexcomm7 {
    // todo
}

// 14 is SPI only
impl Flexcomm for Flexcomm14 {
    // todo
}

// 15 is I2C only
impl Flexcomm for Flexcomm15 {
    // todo
}
