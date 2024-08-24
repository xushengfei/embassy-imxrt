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

/// Flexcomm
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Flexcomm {
    Flexcomm0,
    Flexcomm1,
    Flexcomm2,
    Flexcomm3,
    Flexcomm4,
    Flexcomm5,
    Flexcomm6,
    Flexcomm7,
    Flexcomm14,
    Flexcomm15,
}

/// Flexcomm Config structure, containing:
/// flexcomm: enumeration n
/// function: SPI, UART, I2C from svd
/// lock: whether or not to lock the pselid
#[non_exhaustive]
pub struct Config {
    flexcomm: Flexcomm, // specify which FCn to use
    function: Function, // serial comm peripheral type
    lock: FlexcommLock, // lock the FC, or not
                        // TBD: Specify preferred source clock? ex: low speed / high speed / pll / external
}

impl Default for Config {
    fn default() -> Self {
        Config {
            flexcomm: Flexcomm::Flexcomm0,
            function: Function::NoPeriphSelected,
            lock: FlexcommLock::Unlocked,
        }
    }
}

/// FlexcommConnector includes config and other flexcomm state information
pub struct FlexcommConnector {
    config: Config,
    clock_freq: u32,
}

impl FlexcommConnector {
    // TODO: Use new wip clock traits for all methods

    /// new FlexcommConnector
    pub fn new(_config: Config) -> Self {
        match _config.flexcomm {
            // TBD: return error if flexcomm is locked?
            Flexcomm::Flexcomm0 => FlexcommConnector {
                config: _config,
                clock_freq: 0,
            },

            Flexcomm::Flexcomm1 => FlexcommConnector {
                config: _config,
                clock_freq: 0,
            },

            // TODO: Add for other flexcomm n connectors.
            _ => FlexcommConnector {
                config: _config,
                clock_freq: 0,
            },
        }
    }

    // TBD: Does flexcomm own the associated external config and control bits in SYSCON and RST_CTL ?
    //      If flexcomm does own the external config and control bits, then peripheral drivers
    //      must tell flexcomm which source clock to select (add it to Config struct).

    /// enable channel and connect source clock
    pub fn enable(&mut self) {
        // Enable the Flexcomm connector
        self.attach_clock();
        self.enable_clock();
        self.reset_peripheral();
        self.calculate_clock_frequency();
        self.set_reg();
    }

    /// disable channel and disconnect associated source clock
    pub fn disable(&self) {
        // Disable the Flexcomm connector
        self.disable_clock();
        self.detach_clock();
    }

    /// attach associated source clock (SYSCON CLKCTL1_FC1FCLKSEL)
    fn attach_clock(&self) {
        // attach clock source
        match self.config.flexcomm {
            Flexcomm::Flexcomm0 => {
                todo!(); // pending new clock traits
            }

            Flexcomm::Flexcomm1 => {
                todo!(); // pending new clock traits
            }

            // TODO: Add for other flexcomm n connectors
            _ => {}
        }
    }

    /// detach associated source clock
    fn detach_clock(&self) {
        // attach clock source
        match self.config.flexcomm {
            Flexcomm::Flexcomm0 => {
                todo!(); // pending new clock traits
            }

            Flexcomm::Flexcomm1 => {
                todo!(); // pending new clock traits
            }

            // TODO: Add for other flexcomm n connectors
            _ => {}
        }
    }

    /// Enable the source clock (SYSCON CLKCTL1_PSCCTL0)
    fn enable_clock(&self) {
        match self.config.flexcomm {
            Flexcomm::Flexcomm0 => {
                todo!(); // pending new clock traits
            }

            Flexcomm::Flexcomm1 => {
                todo!(); // pending new clock traits
            }

            // TODO: Add for other flexcomm n connectors
            _ => {}
        }
    }

    /// Disable the source clock
    fn disable_clock(&self) {
        // disable the clock matching the config flexcomm
        match self.config.flexcomm {
            Flexcomm::Flexcomm0 => {
                todo!(); // pending new clock traits
            }

            Flexcomm::Flexcomm1 => {
                todo!(); // pending new clock traits
            }

            // TODO: Add for other flexcomm n connectors
            _ => {}
        }
    }

    /// Set clock_freq to actual source clock frequency
    fn calculate_clock_frequency(&mut self) {
        let freq = 0;
        // TODO: Calculate the actual flexcomm freq based on the clock enabled for this channel
        self.clock_freq = freq;
    }

    /// Reset the flexcomm channel RST_CTLn_PSCCTLn
    fn reset_peripheral(&self) {
        todo!();
    }

    /// Set the peripheral function
    fn set_reg(&self) {
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

    fn regs(&self) -> &'static pac::flexcomm0::RegisterBlock {
        match self.config.flexcomm {
            Flexcomm::Flexcomm0 => unsafe { &*(pac::Flexcomm0::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm1 => unsafe { &*(pac::Flexcomm1::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm2 => unsafe { &*(pac::Flexcomm2::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm3 => unsafe { &*(pac::Flexcomm3::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm4 => unsafe { &*(pac::Flexcomm4::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm5 => unsafe { &*(pac::Flexcomm5::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm6 => unsafe { &*(pac::Flexcomm6::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm7 => unsafe { &*(pac::Flexcomm7::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm14 => unsafe { &*(pac::Flexcomm14::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm15 => unsafe { &*(pac::Flexcomm15::ptr() as *const pac::flexcomm0::RegisterBlock) },
        }
    }
}
