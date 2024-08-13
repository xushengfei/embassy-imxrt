//! Clock configuration for the RT6xx
use crate::pac;

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
pub(crate) unsafe fn init(_config: ClockConfig) {
    let cc0 = unsafe { pac::Clkctl0::steal() };
    let cc1 = unsafe { pac::Clkctl1::steal() };
    let r = unsafe { pac::Rtc::steal() };

    // Enable the RTC peripheral clock
    cc1.pscctl2().modify(|_r, w| w.rtc_lite_clk().enable_clock());
    // Make sure the reset bit is cleared
    r.ctrl().modify(|_r, w| w.swreset().clear_bit());
    // Make sure the RTC OSC is powered up
    r.ctrl().modify(|_r, w| w.rtc_osc_pd().clear_bit());
    // set initial match value, note that with a 15 bit count-down timer this would
    // typically be 0x8000, but we are "doing some clever things" in time-driver.rs,
    // read more about it in the comments there
    r.wake().modify(|_r, w| w.bits(0xA));
    cc0.osc32khzctl0().modify(|_r, w| w.ena32khz().set_bit()); // Enable 32K OSC

    // enable rtc clk
    r.ctrl().modify(|_r, w| w.rtc_en().set_bit());
}
