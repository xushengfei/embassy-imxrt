pub(super) trait SealedInstance {
    /// Returns a reference to peripheral's register block.
    fn regs() -> &'static crate::pac::flexcomm2::RegisterBlock;

    /// Initializes power and clocks to peripheral.
    fn init() -> ();
}

/// WWDT instance trait
#[allow(private_bounds)]
pub(super) trait Instance: SealedInstance {}
