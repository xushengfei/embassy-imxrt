//! Clock configuration for the RT6xx
use core::arch::asm;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

//use cortex_m::peripheral::syst::SystClkSource;
//use cortex_m::peripheral::SYST;
use embassy_hal_internal::{into_ref, PeripheralRef};

//use crate::gpio::{AnyPin, SealedPin}; //not written yet
use crate::{interrupt, pac, Peripheral}; //, reset
use cortex_m::Peripherals;

/// Clock configuration;
#[non_exhaustive]
pub struct Clocks {
    lposc: AtomicU32,
    sfro: AtomicU32,
    rtc: AtomicU32,
    ffro: AtomicU32, //div2 and div4 variations
    clk_in: AtomicU32,
    hclk: AtomicU32, //AHB bus clock
    main_clk: AtomicU32,
    main_pll_clk: AtomicU32, //also has aux0,aux1,dsp, and audio pll's downstream
    os_timer_clk: AtomicU32,
    sys_clk: AtomicU32,
    adc: AtomicU32,
}

static CLOCKS: Clocks = Clocks {
    lposc: AtomicU32::new(0),
    sfro: AtomicU32::new(0),
    rtc: AtomicU32::new(0),
    ffro: AtomicU32::new(0),
    clk_in: AtomicU32::new(0),
    hclk: AtomicU32::new(0),
    main_clk: AtomicU32::new(0),
    main_pll_clk: AtomicU32::new(0),
    os_timer_clk: AtomicU32::new(0),
    sys_clk: AtomicU32::new(0),
    adc: AtomicU32::new(0),
};

/// Clock configuration.
#[non_exhaustive]
pub struct ClockConfig {
    /// low-power oscillator
    pub lposc: Option<LposcConfig>,
    /// 16Mhz internal oscillator
    pub sfro: SfroConfig,
    // Real Time Clock
    pub rtc: Option<RtcClkConfig>,
    /// 48/60 Mhz internal oscillator
    pub ffro: Option<FfroConfig>,
    //pub pll: Option<PllPfdConfig>, //potentially covered in main pll clk
    pub clk_in: ClkInConfig,
    /// AHB bus clock
    pub hclk: HclkConfig,
    pub main_clk: Option<MainClkConfig>,
    pub main_pll_clk: Option<MainPllClkConfig>,
    //pub os_timer_clk: Option<OsTimerClkConfig>, //TODO
    /// Software concept to be used with systick, doesn't map to a register
    pub sys_clk: SysClkConfig,
    //pub adc: Option<AdcConfig>, //TODO: add config
}

impl ClockConfig {
    /// Clock configuration derived from external crystal.
    pub fn crystal(_crystal_hz: u32) -> Self {
        Self {
            lposc: Some(LposcConfig {}),
            sfro: SfroConfig {},
            rtc: Some(RtcClkConfig {
                freq: RtcFreq::Default_1Hz,
                rtc_int: Some(RtcInterrupts::None),
            }),
            ffro: Some(FfroConfig {}),
            //pll: Some(PllConfig {}),//includes aux0 and aux1 pll
            clk_in: ClkInConfig {},
            hclk: HclkConfig { div: 0 },
            main_clk: Some(MainClkConfig {
                //FFRO divided by 4 is reset values of Main Clk Sel A, Sel B
                src: MainClkSrc::FFRO,
                div_int: 4,
            }),
            main_pll_clk: Some(MainPllClkConfig {
                src: MainPllClkSrc::SFRO,
                mult: 16,
                pfd0: 19,
                pfd1: 0,
                pfd2: 0,
                pfd3: 0,
                aux0_div: 0,
                aux1_div: 0,
            }),
            //os_timer_clk: Some(OsTimerClkConfig {}),
            sys_clk: SysClkConfig {
                sysclkfreq: 250_000_000, //TODO: Verify, going off gen3 math
            },
            //adc: Some(AdcConfig {}), // TODO: add config
        }
    }
}

/// Low power oscillator
pub struct LposcConfig {}

pub struct SfroConfig {}

pub enum RtcFreq {
    Default_1Hz,
    HighResolution_1kHz,
    SubSecond_32kHz,
}

pub enum RtcInterrupts {
    None,
    Alarm,
    WakeUp,
}
/// RTC clock config.
pub struct RtcClkConfig {
    /// RTC clock source.
    pub freq: RtcFreq,
    /// RTC Interrupt
    pub rtc_int: Option<RtcInterrupts>,
}

pub struct FfroConfig {}

/// PLL clock source
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainPllClkSrc {
    /// SFRO
    SFRO,
    /// External Clock
    ClkIn,
    // FFRO
    FFRO,
}

/// PLL configuration.
pub struct MainPllClkConfig {
    /// Main clock source.
    pub src: MainPllClkSrc,
    //TODO: missing numerator and denominator?
    /// Multiplication factor.
    pub mult: u8,
    // the following are actually 6-bits not 8
    pub pfd0: u8,
    pub pfd1: u8,
    pub pfd2: u8,
    pub pfd3: u8,
    // Aux dividers
    pub aux0_div: u8,
    pub aux1_div: u8,
}

pub struct ClkInConfig {}

pub struct HclkConfig {
    // divider to turn main clk into hclk for AHB bus
    pub div: u8,
    // TODO: Clock gating?
}

/// Main clock source.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainClkSrc {
    /// FFRO divided by 4
    //FFROdiv4, // probably don't need since it'll be covered by div_int
    /// External Clock
    ClkIn,
    // Low Power Oscillator
    Lposc,
    // FFRO
    FFRO,
    // SFRO
    SFRO,
    // Main PLL Clock
    PllMain,
    /// RTC 32kHz oscillator.
    RTC32k,
}

/// Main clock config.
pub struct MainClkConfig {
    /// Main clock source.
    pub src: MainClkSrc,
    /// Main clock divider.
    pub div_int: u32,
}
/// OS Timer Clk Config
//pub struct OsTimerClkConfig{}

pub struct SysClkConfig {
    pub sysclkfreq: u32,
}

/// ADC clock source.
//TODO

/// ADC clock config.
//TODO

fn clock_ctrls() -> (
    &'static pac::clkctl0::RegisterBlock,
    &'static pac::clkctl1::RegisterBlock,
) {
    unsafe { (&*pac::Clkctl0::ptr(), &*pac::Clkctl1::ptr()) }
}
fn rtc() -> &'static pac::rtc::RegisterBlock {
    unsafe { &*pac::Rtc::ptr() }
}
fn timer0() -> &'static pac::ctimer0::RegisterBlock {
    unsafe { &*pac::Ctimer0::ptr() }
}

/*unsafe fn systick() -> &'static cortex_m::peripheral::SYST {
    //unsafe { &*cortex_m::peripheral::SYST::PTR}
    &*(cortex_m::Peripherals::steal().SYST)
}

pub fn enable_systick() {
    let mut systickRegs = unsafe{systick()};
    systickRegs.disable_interrupt();
    systickRegs.set_clock_source(SystClkSource::Core);
    systickRegs.set_reload(0x800000);
    systickRegs.clear_current();
    systickRegs.enable_counter();
    //let mut syst = cortex_m::peripheral::SYST;
    //syst.
    // Just the RTC
    //enable timer reset on int and interrupts
    /*let r = rtc();
    let t0 = timer0();
    // TODO: should we clear on int if we're using the same counter for
    t0.mcr().modify(|_r, w| {
        w.mr0i()
            .set_bit()
            .mr0r()
            .set_bit()
            .mr1i()
            .set_bit()
            .mr1r()
            .set_bit()
            .mr2i()
            .set_bit()
            .mr2r()
            .set_bit()
            .mr3i()
            .set_bit()
            .mr3r()
            .set_bit()
    });*/
}

pub fn get_systick_count() -> u32 {
    let mut systickRegs = unsafe{systick()};
    let count = systickRegs.cvr.read().try_into();//?
}
pub fn enable_systick_int() {
    let systickRegs = unsafe{systick()};
    systickRegs.enable_interrupt();
}*/
/// safety: must be called exactly once at bootup
pub(crate) unsafe fn init(_config: ClockConfig) {
    let (cc0, cc1) = clock_ctrls();
    let r = rtc();

    //enable_systick();
    cc1.pscctl2()
        .modify(|_r, w| w.rtc_lite_clk().enable_clock()); // Enable the RTC peripheral clock
    r.ctrl().modify(|_r, w| w.swreset().clear_bit()); // Make sure the reset bit is cleared
    r.ctrl().modify(|_r, w| w.rtc_osc_pd().clear_bit()); // Make sure the RTC OSC is powered up
    r.wake().modify(|_r, w| w.bits(0xA));//set initial match value, w.bits(0x8000
    cc0.osc32khzctl0().modify(|_r, w| w.ena32khz().set_bit()); // Enable 32K OSC

    //enable rtc clk
    r.ctrl().modify(|_r, w| w.rtc_en().set_bit());

}