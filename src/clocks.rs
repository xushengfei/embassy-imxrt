//! Clock configuration for the RP2040

/// Clock configuration;
#[non_exhaustive]
pub struct ClockConfig {}

impl ClockConfig {
    /// Clock configuration derived from external crystal.
    pub fn crystal(_crystal_hz: u32) -> Self {
        Self {}
    }
}

/// safety: must be called exactly once at bootup
pub(crate) unsafe fn init(_config: ClockConfig) {}
