//! I2C (Inter-Integrated Circuit) bus Flexcomm Peripheral Setup

use super::instance::{Instance, SealedInstance};

// Cortex-M33 Flexcomm2
impl SealedInstance for crate::peripherals::FLEXCOMM2 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm2::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm2::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c2::RegisterBlock {
        unsafe { &*crate::pac::I2c2::ptr() }
    }

    fn init() {
        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc2_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(2).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc2_clk_set().set_bit());

        let flexcomm = Self::flexcomm_regs();
        let pselid = flexcomm.pselid();

        // Check I2C Support
        if pselid.read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm2");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}
impl Instance for crate::peripherals::FLEXCOMM2 {}
