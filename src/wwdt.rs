//! Windowed Watchdog Timer (WWDT)

use crate::interrupt;
use crate::pac::Interrupt;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use cortex_m::peripheral::NVIC;
use embassy_hal_internal::Peripheral;
use embassy_sync::waitqueue::AtomicWaker;

// Tracks state of watchdog warning futures
const WWDT_COUNT: usize = 2;
const NEW_AW: AtomicWaker = AtomicWaker::new();
const NEW_AB: AtomicBool = AtomicBool::new(false);
static WWDT_WAKERS: [AtomicWaker; WWDT_COUNT] = [NEW_AW; WWDT_COUNT];
static WWDT_WARNINGS: [AtomicBool; WWDT_COUNT] = [NEW_AB; WWDT_COUNT];

/// Windowed watchdog timer (WWDT) driver.
pub struct WindowedWatchdog<'d, T: Instance, M: Mode> {
    _wwdt: PhantomData<&'d mut T>,
    _mode: PhantomData<M>,
}

trait SealedInstance {
    /// Peripheral's instance number.
    const INST: usize;

    /// Returns a reference to peripheral's register block.
    fn regs() -> &'static crate::pac::wwdt0::RegisterBlock;

    /// Initializes power and clocks to peripheral.
    fn init();

    /// Disables peripheral when going out-of-scope.
    ///
    /// If the peripheral was previously locked, only interrupts
    /// are disabled as the clock can no longer be disabled by software.
    fn drop();
}

/// WWDT instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance {}

// Cortex-M33 watchdog
impl SealedInstance for crate::peripherals::WDT0 {
    const INST: usize = 0;

    fn regs() -> &'static crate::pac::wwdt0::RegisterBlock {
        unsafe { &*crate::pac::Wwdt0::ptr() }
    }

    fn init() {
        init_lposc();

        // Enable WWDT0 clock and set LPOSC as clock source
        let clkctl0 = unsafe { &*crate::pac::Clkctl0::ptr() };
        clkctl0.pscctl2_set().write(|w| w.wwdt0_clk().set_bit());
        clkctl0
            .wdt0fclksel()
            .modify(|_, w| unsafe { w.sel().bits(0) });

        // Clear WWDT0 peripheral reset
        let rstctl0 = unsafe { &*crate::pac::Rstctl0::ptr() };
        rstctl0.prstctl2_clr().write(|w| w.wwdt0().set_bit());

        // Allow WDT0 interrupts to wake device from deep-sleep mode
        let sysctl0 = unsafe { &*crate::pac::Sysctl0::ptr() };
        sysctl0.starten0_set().write(|w| w.wdt0().set_bit());

        // Enable interrupts
        unsafe { NVIC::unmask(Interrupt::WDT0) };
    }

    fn drop() {
        // Disable interrupt
        NVIC::mask(Interrupt::WDT0);

        // Disable watchdog clock
        let clkctl0 = unsafe { &*crate::pac::Clkctl0::ptr() };
        clkctl0.pscctl2_clr().write(|w| w.wwdt0_clk().set_bit());
    }
}
impl Instance for crate::peripherals::WDT0 {}

// HiFi4 DSP watchdog
impl SealedInstance for crate::peripherals::WDT1 {
    const INST: usize = 1;

    fn regs() -> &'static crate::pac::wwdt0::RegisterBlock {
        unsafe { &*crate::pac::Wwdt1::ptr() }
    }

    fn init() {
        init_lposc();

        // Enable WWDT1 clock and set LPOSC as clock source
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };
        clkctl1.pscctl2_set().write(|w| w.wwdt1_clk_set().set_bit());
        clkctl1
            .wdt1fclksel()
            .modify(|_, w| unsafe { w.sel().bits(0) });

        // Clear WWDT1 peripheral reset
        let rstctl1 = unsafe { &*crate::pac::Rstctl1::ptr() };
        rstctl1
            .prstctl2_clr()
            .write(|w| w.wwdt1_rst_clr().set_bit());

        // Enable interrupts
        unsafe { NVIC::unmask(Interrupt::WDT1) };
    }

    fn drop() {
        // Disable interrupt
        NVIC::mask(Interrupt::WDT1);

        // Disable watchdog clock
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };
        clkctl1.pscctl2_clr().write(|w| w.wwdt1_clk_clr().set_bit());
    }
}
impl Instance for crate::peripherals::WDT1 {}

trait SealedMode {}

/// WWDT mode trait.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

/// Watchdog is leashed and not currently running.
pub struct Leashed;
impl SealedMode for Leashed {}
impl Mode for Leashed {}

/// Watchdog is unleashed and will run permanently until reset.
///
/// Must be fed regularly or else timeout event will occur.
pub struct Unleashed;
impl SealedMode for Unleashed {}
impl Mode for Unleashed {}

// Fixed watchdog clock prescaler
const PSC: u32 = 4;

// Low-power oscillator frequency
const LPOSC_HZ: u32 = 1_000_000;

// Microseconds per low-power oscillator tick
const US_PER_TICK: u32 = 1_000_000 / LPOSC_HZ;

// Minimum time that can be set as watchdog timeout
const MIN_TIMEOUT_US: u32 = US_PER_TICK * 256 * PSC;

// Maximum time that can fit in watchdog counter
const MAX_COUNTER_US: u32 = US_PER_TICK * 16_777_216 * PSC;

// Maximum time that can be set as watchdog warning threshold
const MAX_WARNING_US: u32 = US_PER_TICK * 1024 * PSC;

/// Converts a time in microseconds to a WWDT counter value.
const fn time_to_counter(time_us: u32) -> u32 {
    (time_us / (US_PER_TICK * PSC)) - 1
}

/// Converts a WWDT counter value to a time in microseconds.
const fn counter_to_time(counter: u32) -> u32 {
    (counter + 1) * (US_PER_TICK * PSC)
}

/// Initializes low-power oscillator.
fn init_lposc() {
    // Enable low power oscillator
    let sysctl0 = unsafe { &*crate::pac::Sysctl0::ptr() };
    sysctl0.pdruncfg0_clr().write(|w| w.lposc_pd().set_bit());

    // Wait for low-power oscillator to be ready (typically 64 us)
    // Busy loop seems better here than trying to shoe-in an async delay
    let clkctl0 = unsafe { &*crate::pac::Clkctl0::ptr() };
    while clkctl0.lposcctl0().read().clkrdy().bit_is_clear() {}
}

impl<'d, T: Instance> WindowedWatchdog<'d, T, Leashed> {
    /// Creates a WWDT (Windowed Watchdog Timer) instance with a given timeout value in microseconds.
    ///
    /// [Self] has to be started with [Self::unleash], but should be configured beforehand.
    ///
    /// To enable system reset upon timeout, [Self::enable_reset] must be called,
    /// otherwise only warning interrupts will be generated.
    ///
    /// Because the timeout flag is not cleared by a watchdog reset, this must be manually cleared
    /// by calling [Self::clear_timeout_flag] after creation.
    ///
    /// This is not automatically cleared here because application code may wish to check
    /// if it is set via a call to [Self::timed_out] to determine if a watchdog reset occurred previously.
    pub fn new(_instance: impl Peripheral<P = T> + 'd, timeout_us: u32) -> Self {
        let mut wwdt = Self {
            _wwdt: PhantomData,
            _mode: PhantomData,
        };

        T::init();
        wwdt.set_timeout(timeout_us);
        wwdt
    }

    /// Enables a full system reset upon a watchdog timeout, which cannot be undone until reset occurs.
    pub fn enable_reset(&mut self) -> &mut Self {
        T::regs().mod_().modify(|_, w| w.wdreset().set_bit());
        self
    }

    /// Permanently prevents the watchdog oscillator from being powered down by software until reset.
    pub fn lock(&mut self) -> &mut Self {
        T::regs().mod_().modify(|_, w| w.lock().set_bit());
        self
    }

    /// Sets the window in microseconds before a timeout that watchdog feeds are allowed.
    ///
    /// Attempting a feed outside this window will cause a watchdog event to occur.
    ///
    /// On reset, the feed window equals the max possible timeout value, thus windowing
    /// is effectively disabled.
    pub fn set_feed_window(&mut self, window_us: u32) -> &mut Self {
        debug_assert!((0..=MAX_COUNTER_US).contains(&window_us));
        let counter = time_to_counter(window_us);
        T::regs()
            .window()
            .write(|w| unsafe { w.window().bits(counter) });
        self
    }

    /// Sets the threshold in microseconds before a timeout below which a warning interrupt will be generated.
    ///
    /// If warning interrupt occurs, the warning flag must be manually cleared
    /// via a call to [Self::clear_warning_flag].
    pub fn set_warning_threshold(&mut self, threshold_us: u32) -> &mut Self {
        debug_assert!((0..=MAX_WARNING_US).contains(&threshold_us));
        let counter = time_to_counter(threshold_us) as u16;
        T::regs()
            .warnint()
            .write(|w| unsafe { w.warnint().bits(counter) });
        self
    }

    /// Permanently prevents the watchdog timeout counter from being changed until reset
    /// unless the counter is below the warning and feed window thresholds.
    /// Attempting to do so will cause a watchdog timeout event.
    ///
    /// However, a call to [Self::set_timeout] alone will not cause a watchdog timeout event,
    /// [Self::feed] must be called as well.
    pub fn protect_timeout(&mut self) -> &mut Self {
        T::regs().mod_().modify(|_, w| w.wdprotect().set_bit());
        self
    }

    /// Starts the watchdog timer, which cannot be stopped until a system reset.
    ///
    /// [Self::feed] must be called periodically to prevent a timeout event from occurring.
    ///
    /// Most configuration (such as setting thresholds/feed windows, locking/protecting, etc)
    /// must be performed before this call.
    pub fn unleash(self) -> WindowedWatchdog<'d, T, Unleashed> {
        T::regs().mod_().modify(|_, w| w.wden().set_bit());

        // Our destructor disables the watchdog peripheral, but we don't want that here.
        // So we take ownership of self WITHOUT calling drop().
        core::mem::forget(self);

        let mut unleashed_wwdt = WindowedWatchdog {
            _wwdt: PhantomData,
            _mode: PhantomData,
        };

        unleashed_wwdt.feed();
        unleashed_wwdt
    }
}

impl<'d, T: Instance> WindowedWatchdog<'d, T, Unleashed> {
    /// Reloads the watchdog timeout counter to the time set by [Self::set_timeout].
    pub fn feed(&mut self) {
        /* Disable interrupts to prevent possibility of watchdog registers from being accessed in between
         * writes of feed sequence bytes as per datasheet's recommendation.
         */
        critical_section::with(|_| {
            [0xAA, 0x55]
                .iter()
                .for_each(|byte| T::regs().feed().write(|w| unsafe { w.feed().bits(*byte) }))
        });
    }

    /// Asynchronously wait for watchdog warning interrupt to be generated.
    ///
    /// Can be used to handle last-millisecond tasks before system reset occurs.
    pub async fn wait_for_warning(&mut self) {
        WatchdogFuture::<T>::new().await
    }
}

impl<'d, T: Instance, M: Mode> WindowedWatchdog<'d, T, M> {
    /// Returns true if the warning flag is set.
    ///
    /// Flag is set if watchdog timeout counter has fallen below the time
    /// set by a previous call to [Self::set_warning_threshold].
    ///
    /// Must be manually cleared with a call to [Self::clear_warning_flag].
    pub fn warning(&self) -> bool {
        T::regs().mod_().read().wdint().bit_is_set()
    }

    /// Clears the warning interrupt flag.
    pub fn clear_warning_flag(&mut self) {
        // Warning flag is cleared by writing a 1
        T::regs().mod_().modify(|_, w| w.wdint().set_bit());
    }

    /// Returns the time in microseconds until a watchdog timeout event will occur.
    pub fn timeout(&self) -> u32 {
        let counter = T::regs().tv().read().count().bits();
        counter_to_time(counter)
    }

    /// Sets the time in microseconds before a watchdog timeout occurs.
    ///
    /// [Self::feed] must still be called to reload the watchdog timer.
    ///
    /// If [Self::protect_timeout] has been previously called, calling this method
    /// will cause a watchdog timeout event if counter is above the
    /// warning or feed window thresholds and a [Self::feed] call is made.
    pub fn set_timeout(&mut self, timeout_us: u32) {
        debug_assert!((MIN_TIMEOUT_US..=MAX_COUNTER_US).contains(&timeout_us));
        let counter = time_to_counter(timeout_us);
        T::regs().tc().write(|w| unsafe { w.count().bits(counter) })
    }

    /// Returns true if the watchdog timeout flag is set.
    ///
    /// Flag is set if a watchdog timeout event occurs,
    /// and is not automatically cleared on a watchdog reset.
    ///
    /// This allows application to check if a watchdog reset has just occurred.
    ///
    /// Must be manually cleared with a call to [Self::clear_timeout_flag].
    pub fn timed_out(&self) -> bool {
        T::regs().mod_().read().wdtof().bit_is_set()
    }

    /// Clears the watchdog timeout flag.
    pub fn clear_timeout_flag(&mut self) {
        T::regs().mod_().modify(|_, w| w.wdtof().clear_bit());
    }

    /// Returns the current feed window in microseconds.
    pub fn feed_window(&self) -> u32 {
        let counter = T::regs().window().read().window().bits();
        counter_to_time(counter)
    }

    /// Returns the current warning threshold in microseconds.
    pub fn warning_threshold(&self) -> u32 {
        let counter = T::regs().warnint().read().warnint().bits();
        counter_to_time(counter as u32)
    }
}

impl<'d, T: Instance, M: Mode> Drop for WindowedWatchdog<'d, T, M> {
    fn drop(&mut self) {
        T::drop();
    }
}

struct WatchdogFuture<'d, T: Instance> {
    _wwdt: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> WatchdogFuture<'d, T> {
    fn new() -> Self {
        Self { _wwdt: PhantomData }
    }
}

impl<'d, T: Instance> Future for WatchdogFuture<'d, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        WWDT_WAKERS[T::INST].register(cx.waker());

        if WWDT_WARNINGS[T::INST].load(Ordering::Acquire) {
            WWDT_WARNINGS[T::INST].store(false, Ordering::Release);
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

macro_rules! wwdt_isr {
    ($n:expr, $WDTx:ident, $Wwdtx:ident) => {
        #[allow(non_snake_case)]
        #[interrupt]
        fn $WDTx() {
            let wwdt = unsafe { &*$crate::pac::$Wwdtx::ptr() };
            wwdt.mod_().modify(|_, w| w.wdint().set_bit());
            WWDT_WARNINGS[$n].store(true, Ordering::Release);
            WWDT_WAKERS[$n].wake();
        }
    };
}

wwdt_isr!(0, WDT0, Wwdt0);
wwdt_isr!(1, WDT1, Wwdt1);
