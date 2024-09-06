//!USART settings - temporary file until the clocks and flexcomm are implemented.
//!

#![macro_use]

use crate::pac::flexcomm0;
use crate::pac::Clkctl0;
use crate::pac::Clkctl1;
use crate::uart::GenericStatus;
use mimxrt685s_pac as pac;

// Re-export SVD variants to allow user to directly set values.
pub use pac::flexcomm0::pselid::Lock as FlexcommLock;
pub use pac::flexcomm0::pselid::Persel as Function;

pub enum FlexcommFunc {
    // Only define the first one for now.
    Flexcomm0,
}

pub struct Flexcomm {
    flexcomm: FlexcommFunc,
    function: Function,
    lock: FlexcommLock,
}

impl Flexcomm {
    pub fn new() -> Self {
        // hardcoding the config for now
        Self {
            flexcomm: FlexcommFunc::Flexcomm0,
            function: Function::Usart,
            lock: FlexcommLock::Unlocked,
        }
    }

    pub fn init(&self) {
        self.clock_attach();
        self.clock_enable();
        self.reset_peripheral();
        let mut status = self.flexcomm_set_peripheral();
        if status != GenericStatus::Success {
            info!("Error: Flexcomm peripheral not supported");
        }
    }

    pub fn flexcomm_getClkFreq(&self) -> u32 {
        // Get the clock frequency of the flexcomm
        // For now, hardcoding the value for flexcomm0
        let freq = self.clock_get_audio_pll_clk_freq();
        return freq;
    }

    fn clock_get_audio_pll_clk_freq(&self) -> u32 {
        // return CLOCK_GetAudioPfdFreq(kCLOCK_Pfd0) / ((CLKCTL1->AUDIOPLLCLKDIV & CLKCTL1_AUDIOPLLCLKDIV_DIV_MASK) + 1U);
        // TODO: check and hardcode.
        return 20000000; //0x2dc6c00;
    }

    /// Exposing a method to access reg internally with the assumption that only the flexcomm0 is being used
    fn reg(&self) -> &'static pac::flexcomm0::RegisterBlock {
        unsafe { &*(pac::Flexcomm0::ptr() as *const pac::flexcomm0::RegisterBlock) }
    }

    /// Exposing a method to access reg internally with the assumption that only the clkctl1 is being used
    fn clk1_reg(&self) -> &'static pac::clkctl1::RegisterBlock {
        unsafe { &*(pac::Clkctl1::ptr() as *const pac::clkctl1::RegisterBlock) }
    }

    /// Exposing a method to access reg internally with the assumption that only the peripheral reset control 1 is being used
    fn rstctl1_reg(&self) -> &'static pac::rstctl1::RegisterBlock {
        unsafe { &*(pac::Rstctl1::ptr() as *const pac::rstctl1::RegisterBlock) }
    }

    /// This func would return the specific instance of the flexcomm peripheral, i.e flexcomm0,1, 2,etc
    /*fn get_instance() -> u32 {
        // add code
        return 0;
    }*/

    fn clock_attach(&self) {
        // Be careful to connect the correct clock.
        // In gen3, this func deals with this CLOCK_AttachClk()
        // For the purpose of uart testing, the following case is hardcoded :
        // CLOCK_AttachClk(mcuPortDef_FlexCommAudioClkSelect[eFLEXCOMM_DEBUG_UART])
        // kAUDIO_PLL_to_FLEXCOMM0  = CLKCTL1_TUPLE_MUXA(FC0FCLKSEL_OFFSET, 2), => [( 0x80000000U | (0x508 | 0x2000))= 0x80002508 ]
        // So pClkSet will be 0x4002 1508 => FC0FCLKSEL (full CLKCTL1_FC0FCLKSEL)

        /*self.clk1_reg()
        .flexcomm(0)
        .fcfclksel()
        .write(|w| w.sel().audio_pll_clk());*/
        self.clk1_reg()
            .flexcomm(0)
            .fcfclksel()
            .modify(|_, w| w.sel().audio_pll_clk());
    }

    fn clock_enable(&self) {
        // Enable the peripheral clock
        // kCLOCK_Flexcomm0    = CLK_GATE_DEFINE(CLK_CTL1_PSCCTL0, 8)
        //Note: modify doesnt exist for pscctl0_set() because its a write only register
        self.clk1_reg().pscctl0_set().write(|w| w.fc0_clk_set().set_bit());
    }

    fn reset_peripheral(&self) {
        // Reset the FLEXCOMM module
        //Note: modify doesnt exist for prstctl0() because its a write only register
        self.rstctl1_reg().prstctl0().write(|w| w.flexcomm0_rst().set_bit());
        self.rstctl1_reg().prstctl0().write(|w| w.flexcomm0_rst().clear_bit());
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

        //self.reg().pselid().write(|w| w.persel().usart());
        self.reg().pselid().modify(|_, w| w.persel().usart());
        if self.lock == FlexcommLock::Locked {
            // Lock the Flexcomm to the peripheral type
            //self.reg().pselid().write(|w| w.lock().set_bit());
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
