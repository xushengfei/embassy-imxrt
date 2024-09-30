//!USART settings - temporary file until the clocks and flexcomm are implemented.
//!

#![macro_use]

use crate::iopctl::Function as PinFunction;
use crate::iopctl::*;
use crate::pac::flexcomm0;
use crate::pac::Clkctl1;
use crate::uart::GenericStatus;
use mimxrt685s_pac as pac;

// Re-export SVD variants to allow user to directly set values.
pub use pac::flexcomm0::pselid::Lock as FlexcommLock;
pub use pac::flexcomm0::pselid::Persel as Function;

#[derive(Copy, Clone)]
pub enum FlexcommFunc {
    // Only define the first one for now.
    //Flexcomm0,
    Flexcomm1,
    Flexcomm2,
}

pub struct Flexcomm {
    flexcomm: FlexcommFunc,
    function: Function,
    lock: FlexcommLock,
}

impl Flexcomm {
    pub fn new(fc: FlexcommFunc) -> Self {
        // hardcoding the config for now
        Self {
            flexcomm: fc,
            function: Function::Usart,
            lock: FlexcommLock::Unlocked,
        }
    }

    pub fn init(&self) {
        self.clock_enable();
        self.clock_attach();
        self.reset_peripheral();
        let mut status = self.flexcomm_set_peripheral();
        if status != GenericStatus::Success {
            info!("Error: Flexcomm peripheral not supported");
        }
    }

    /// Exposing a method to access reg internally with the assumption that only the flexcomm0 is being used
    fn reg(&self) -> &'static pac::flexcomm0::RegisterBlock {
        match self.flexcomm {
            FlexcommFunc::Flexcomm1 => {
                return unsafe { &*(pac::Flexcomm1::ptr() as *const pac::flexcomm0::RegisterBlock) }
            }
            FlexcommFunc::Flexcomm2 => {
                return unsafe { &*(pac::Flexcomm2::ptr() as *const pac::flexcomm0::RegisterBlock) }
            }
        }
        //unsafe { &*(pac::Flexcomm2::ptr() as *const pac::flexcomm0::RegisterBlock) }
    }

    /// Exposing a method to access reg internally with the assumption that only the clkctl1 is being used
    fn clk1_reg(&self) -> &'static pac::clkctl1::RegisterBlock {
        unsafe { &*(pac::Clkctl1::ptr() as *const pac::clkctl1::RegisterBlock) }
    }

    /// Exposing a method to access reg internally with the assumption that only the peripheral reset control 1 is being used
    fn rstctl1_reg(&self) -> &'static pac::rstctl1::RegisterBlock {
        unsafe { &*(pac::Rstctl1::ptr() as *const pac::rstctl1::RegisterBlock) }
    }

    fn clock_attach(&self) {
        let mut fc_index: usize = 0;
        match self.flexcomm {
            FlexcommFunc::Flexcomm1 => {
                fc_index = 1;
            }
            FlexcommFunc::Flexcomm2 => {
                fc_index = 2;
            }
        }
        self.clk1_reg()
            .flexcomm(fc_index) //.flexcomm(0)
            .fcfclksel()
            .modify(|_, w| w.sel().sfro_clk()); //.modify(|_, w| w.sel().audio_pll_clk());
        self.clk1_reg()
            .flexcomm(fc_index)
            .frgclksel()
            .write(|w| w.sel().sfro_clk());
        unsafe {
            self.clk1_reg()
                .flexcomm(fc_index)
                .frgctl()
                .modify(|_, w| w.div().bits(0xff).mult().bits(0));
        }
    }

    fn clock_enable(&self) {
        // Enable the peripheral clock
        match self.flexcomm {
            FlexcommFunc::Flexcomm1 => self.clk1_reg().pscctl0_set().write(|w| w.fc1_clk_set().set_clock()),
            FlexcommFunc::Flexcomm2 => self.clk1_reg().pscctl0_set().write(|w| w.fc2_clk_set().set_clock()),
        }
        //self.clk1_reg().pscctl0_set().write(|w| w.fc2_clk_set().set_clock());
    }

    fn reset_peripheral(&self) {
        // Reset the FLEXCOMM module
        match self.flexcomm {
            FlexcommFunc::Flexcomm1 => self.rstctl1_reg().prstctl0().write(|w| w.flexcomm1_rst().clear_reset()),
            FlexcommFunc::Flexcomm2 => self.rstctl1_reg().prstctl0().write(|w| w.flexcomm2_rst().clear_reset()),
        }
        //self.rstctl1_reg().prstctl0().write(|w| w.flexcomm2_rst().clear_reset());
    }

    fn flexcomm_set_peripheral(&self) -> GenericStatus {
        // Set the FLEXCOMM to given peripheral

        if self.flexcomm_is_peripheral_supported() == false {
            return GenericStatus::NoTransferInProgress;
        }

        // Check if Flexcomm is locked to different peripheral type than expected
        if self.reg().pselid().read().lock().is_locked() && (self.reg().pselid().read().persel().is_usart() == false) {
            return GenericStatus::Fail;
        }

        self.reg().pselid().modify(|_, w| w.persel().usart());
        if self.lock == FlexcommLock::Locked {
            // Lock the Flexcomm to the peripheral type
            self.reg().pselid().modify(|_, w| w.lock().set_bit());
        }

        return GenericStatus::Success;
    }

    /// This function checks whether flexcomm supports peripheral type
    fn flexcomm_is_peripheral_supported(&self) -> bool {
        // Check if the peripheral is supported by the flexcomm
        // TODO: Check for all peripheral types. For now only usart being supported by flexcomm 0 is checked.
        let mut is_usart_present = self.reg().pselid().read().usartpresent().bit_is_set();
        return is_usart_present;
    }
}
