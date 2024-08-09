use super::instance::{Instance, SealedInstance};

// Cortex-M33 Flexcomm2
impl SealedInstance for crate::peripherals::FLEXCOMM2 {
    fn regs() -> &'static crate::pac::flexcomm2::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm2::ptr() }
    }

    fn init() {}
}
impl Instance for crate::peripherals::FLEXCOMM2 {}
