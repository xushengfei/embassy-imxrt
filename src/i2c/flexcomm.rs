use super::instance::{Instance, SealedInstance};

// Cortex-M33 Flexcomm2
impl SealedInstance for crate::peripherals::FLEXCOMM2 {
    fn regs() -> &'static crate::pac::flexcomm2::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm2::ptr() }
    }

    fn init() {
        // From Section 21.4 for Flexcomm in User Manual, enable fc2_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let pscctl0 = clkctl1.pscctl0();
        pscctl0.modify(|_, w| w.fc2_clk().set_bit());
    }
}
impl Instance for crate::peripherals::FLEXCOMM2 {}
