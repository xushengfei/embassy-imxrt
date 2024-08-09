//! I2C (Inter-Integrated Circuit) bus register wrapper

pub(super) trait SealedInstance {
    /// Returns a reference to peripheral's register block.
    fn flexcomm_regs() -> &'static crate::pac::flexcomm2::RegisterBlock;

    fn i2c_regs() -> &'static crate::pac::i2c2::RegisterBlock;

    /// Initializes power and clocks to peripheral.
    fn init() -> ();
}

/// WWDT instance trait
#[allow(private_bounds)]
pub(super) trait Instance: SealedInstance {}
