//!FLEXCOMM
//!
#![macro_use]

use core::ptr;

use crate::pac::flexcomm0;
use crate::peripherals;
use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{impl_peripheral, interrupt, into_ref, Peripheral, PeripheralRef};
use mimxrt685s_pac as pac;

/// Flexcomm
#[derive(Clone, Copy, Debug, PartialEq)]

/// TODO: Temporary definition of AnyPin. Should be removed after gpio integration
pub struct AnyPin {
    pin_port: u8,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Flexcomm {
    Flexcomm0,
    Flexcomm1,
    Flexcomm2,
    Flexcomm3,
    Flexcomm4,
    Flexcomm5,
    Flexcomm14,
    Flexcomm15,
}
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Function {
    USART,
    SPI,
    I2C,
    I2S, // TODO: Add I2S trans and receive separately
}

pub struct Config {
    flexcomm: Flexcomm,
    function: Function,
    islocked: bool,
}

pub enum ConfigError {
    InvalidConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            flexcomm: Flexcomm::Flexcomm0,
            function: Function::USART,
            islocked: false,
        }
    }
}

impl Config {
    pub fn new(_flexcomm: Flexcomm, _function: Function, _locked: bool) -> Self {
        Config {
            flexcomm: _flexcomm,
            function: _function,
            islocked: _locked,
        }
    }
}

pub struct FlexcommConnector {
    clock_id: u32,
    clock_name: u32,
    flexcomm_sys_reset_reg: u32, // TODO: replace with actual registers
    clock_freq: u32,
    config: Config,
}

impl FlexcommConnector {
    // TODO: Need access to clock control reg, clock IDs, clock name for Flexcomm
    // Check with clock implementation

    pub fn new(_config: Config) -> Self {
        match _config.flexcomm {
            Flexcomm::Flexcomm0 => FlexcommConnector {
                clock_id: 0,
                clock_name: 0,
                flexcomm_sys_reset_reg: 0,
                clock_freq: 0,
                config: _config,
            },

            // TODO: Add for other flexcomm connectors. Check with the clock implementation
            _ => FlexcommConnector {
                clock_id: 0,
                clock_name: 0,
                flexcomm_sys_reset_reg: 0,
                clock_freq: 0,
                config: _config,
            },
        }
    }

    pub fn enable(&mut self) {
        // Enable the Flexcomm connector
        self.attach_clock();
        self.enable_clock();
        self.reset_peripheral();
        self.calculate_clock_frequency();
        self.set_reg();
    }

    pub fn disable(&self) {
        // Disable the Flexcomm connector
        self.disable_clock();
        self.attach_clock();
    }

    fn attach_clock(&self) {
        // Set the clock
    }

    fn enable_clock(&self) {
        // Enable the clock
    }

    fn disable_clock(&self) {
        // Enable the clock
    }

    fn calculate_clock_frequency(&mut self) {
        // TODO: Calculate the flex comm freq and update
        let freq = 0;
        self.clock_freq = freq;
    }

    fn reset_peripheral(&self) {
        // Reset the peripheral
    }

    fn set_reg(&self) {
        // Set the peripheral function

        // TODO: Check if peripheral is present
        // TODO: Check if peripheral is locked and mapped to a diff peripheral

        match self.config.function {
            Function::USART => {
                // Set the peripheral function to USART
                self.regs().pselid().write(|w| w.persel().usart());
            }
            Function::SPI => {
                // Set the peripheral function to SPI
                self.regs().pselid().write(|w| w.persel().spi());
            }
            Function::I2C => {
                // Set the peripheral function to I2C
                self.regs().pselid().write(|w| w.persel().i2c());
            }
            Function::I2S => {
                // Set the peripheral function to I2S
                self.regs().pselid().write(|w| w.persel().i2s_receive());
            }
        }
        if self.config.islocked {
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
            Flexcomm::Flexcomm14 => unsafe { &*(pac::Flexcomm14::ptr() as *const pac::flexcomm0::RegisterBlock) },
            Flexcomm::Flexcomm15 => unsafe { &*(pac::Flexcomm15::ptr() as *const pac::flexcomm0::RegisterBlock) },
        }
    }
}
