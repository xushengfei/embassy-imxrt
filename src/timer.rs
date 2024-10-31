use crate::interrupt;
use crate::pac::Clkctl1;
use crate::pac::Rstctl1;
use crate::pac::{Ctimer0, Ctimer1, Ctimer2, Ctimer3, Ctimer4, Inputmux};
use core::future::poll_fn;
use core::task::Poll;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

const COUNT_CHANNEL: usize = 20;
const CAPTURE_CHANNEL: usize = 20;
const TOTAL_CHANNELS: usize = COUNT_CHANNEL + CAPTURE_CHANNEL;
const CHANNEL_PER_MODULE: usize = 4;

static WAKERS: [AtomicWaker; TOTAL_CHANNELS] = [const { AtomicWaker::new() }; TOTAL_CHANNELS];

pub use embedded_hal_02::timer::{Cancel, CountDown, Periodic};

enum TimerType {
    Counting,
    Capture,
}

#[derive(PartialEq)]
/// Enum representing the edge type for capture channels.
pub enum CaptureChEdge {
    /// Rising edge
    Rising,
    /// Falling edge
    Falling,
    /// Both edges
    Duel,
}

#[derive(Copy, Clone)]
struct Channel {
    allocated: bool,
}

impl Channel {
    fn new() -> Self {
        Self { allocated: false }
    }
}

/// Trait representing a timer that can be started and provides an ID.
pub trait Timer {
    /// Starts the timer with the specified count.
    ///
    /// # Arguments
    ///
    /// * `count` - The countdown value to start the timer with.
    fn start_count(&mut self, count: u32);

    /// Starts the capture timer with the event for capture passed as argument.
    /// # Arguments
    /// * `event_input` - The event input to capture the timer counter.
    fn start_capture(&self, event_input: u32);

    /// Waits for the countdown timer to complete.
    async fn wait(&self);
}

/// A timer that captures events based on a specified edge and calls a user-defined callback.
pub struct CaptureTimer<F: Fn(u32)> {
    _id: usize,
    _clk_freq: u32,
    _cb: F, // User callback closure
    _timeout: u32,
    _edge: CaptureChEdge,
    _periodic: bool,
}

struct CountingTimer<F: Fn()> {
    _id: usize,
    _clk_freq: u32,
    _cb: F, // User callback closure
    _timeout: u32,
    _periodic: bool,
}

/// Interrupt handler for the CTimer modules.
pub struct CtimerInterruptHandler;

macro_rules! irq_handler_impl {
    ($timer:ident, $waker0:expr, $waker1:expr, $waker2:expr, $waker3:expr, $waker4:expr, $waker5:expr, $waker6:expr, $waker7:expr) => {
        let reg = unsafe { $timer::steal() };
        if reg.ir().read().mr0int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr0i().clear_bit());
            reg.ir().modify(|_, w| w.mr0int().set_bit());
            reg.mr(0).write(|w| unsafe { w.match_().bits(0) });
            WAKERS[$waker0].wake();
        }
        if reg.ir().read().mr1int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr1i().clear_bit());
            reg.ir().write(|w| w.mr1int().set_bit());
            reg.mr(1).write(|w| unsafe { w.match_().bits(0) });
            WAKERS[$waker1].wake();
        }
        if reg.ir().read().mr2int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr2i().clear_bit());
            reg.ir().write(|w| w.mr2int().set_bit());
            reg.mr(2).write(|w| unsafe { w.match_().bits(0) });
            WAKERS[$waker2].wake();
        }
        if reg.ir().read().mr3int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr3i().clear_bit());
            reg.ir().write(|w| w.mr3int().set_bit());
            reg.mr(3).write(|w| unsafe { w.match_().bits(0) });
            WAKERS[$waker3].wake();
        }
        if reg.ir().read().cr0int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap0i().clear_bit());
            reg.ir().write(|w| w.cr0int().set_bit());
            WAKERS[$waker4].wake();
        }
        if reg.ir().read().cr1int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap1i().clear_bit());
            reg.ir().write(|w| w.cr1int().set_bit());
            WAKERS[$waker5].wake();
        }
        if reg.ir().read().cr2int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap2i().clear_bit());
            reg.ir().write(|w| w.cr2int().set_bit());
            WAKERS[$waker6].wake();
        }
        if reg.ir().read().cr3int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap3i().clear_bit());
            reg.ir().write(|w| w.cr3int().set_bit());
            WAKERS[$waker7].wake();
        }
    };
}

macro_rules! impl_capture_timer_setup {
    ($timer:ident, $edge:ident, $id:ident) => {
        let reg = unsafe { $timer::steal() };
        let offset = ($id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;

        match offset {
            0 => {
                reg.ccr().modify(|_, w| w.cap0i().set_bit());
                match $edge {
                    CaptureChEdge::Rising => {
                        reg.ccr().modify(|_, w| w.cap0re().set_bit());
                    }
                    CaptureChEdge::Falling => {
                        reg.ccr().modify(|_, w| w.cap0fe().set_bit());
                    }
                    CaptureChEdge::Duel => {
                        reg.ccr().modify(|_, w| w.cap0re().set_bit());
                        reg.ccr().modify(|_, w| w.cap0fe().set_bit());
                    }
                }
            }
            1 => {
                reg.ccr().modify(|_, w| w.cap1i().set_bit());
                match $edge {
                    CaptureChEdge::Rising => {
                        reg.ccr().modify(|_, w| w.cap1re().set_bit());
                    }
                    CaptureChEdge::Falling => {
                        reg.ccr().modify(|_, w| w.cap1fe().set_bit());
                    }
                    CaptureChEdge::Duel => {
                        reg.ccr().modify(|_, w| w.cap1re().set_bit());
                        reg.ccr().modify(|_, w| w.cap1fe().set_bit());
                    }
                }
            }
            2 => {
                reg.ccr().modify(|_, w| w.cap2i().set_bit());
                match $edge {
                    CaptureChEdge::Rising => {
                        reg.ccr().modify(|_, w| w.cap2re().set_bit());
                    }
                    CaptureChEdge::Falling => {
                        reg.ccr().modify(|_, w| w.cap2fe().set_bit());
                    }
                    CaptureChEdge::Duel => {
                        reg.ccr().modify(|_, w| w.cap2re().set_bit());
                        reg.ccr().modify(|_, w| w.cap2fe().set_bit());
                    }
                }
            }
            3 => {
                reg.ccr().modify(|_, w| w.cap3i().set_bit());
                match $edge {
                    CaptureChEdge::Rising => {
                        reg.ccr().modify(|_, w| w.cap3re().set_bit());
                    }
                    CaptureChEdge::Falling => {
                        reg.ccr().modify(|_, w| w.cap3fe().set_bit());
                    }
                    CaptureChEdge::Duel => {
                        reg.ccr().modify(|_, w| w.cap3re().set_bit());
                        reg.ccr().modify(|_, w| w.cap3fe().set_bit());
                    }
                }
            }
            _ => {
                core::panic!("Invalid channel");
            }
        }
    };
}

macro_rules! impl_counting_timer_setup {
    ($timer:ident, $id:ident) => {
        let reg = unsafe { $timer::steal() };
        let offset = $id % CHANNEL_PER_MODULE;

        match offset {
            0 => {
                reg.mcr().modify(|_, w| w.mr0i().set_bit());
            }
            1 => {
                reg.mcr().modify(|_, w| w.mr1i().set_bit());
            }
            2 => {
                reg.mcr().modify(|_, w| w.mr2i().set_bit());
            }
            3 => {
                reg.mcr().modify(|_, w| w.mr3i().set_bit());
            }
            _ => {
                core::panic!("Invalid channel");
            }
        }
    };
}

macro_rules! impl_counting_timer_wait {
    ($timer:ident, $offset: ident, $self:ident) => {
        let reg = unsafe { $timer::steal() };

        if $self._periodic && reg.mr($offset).read().bits() == 0 {
            let cycles = $self._timeout;
            let curr_time = reg.tc().read().bits();

            if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                let cycles = leftover as u32;
                unsafe {
                    reg.mr($offset).write(|w| w.match_().bits(cycles));
                }
            } else {
                unsafe {
                    reg.mr($offset).write(|w| w.match_().bits(curr_time + cycles));
                }
            }
        }

        if $offset == 0 && reg.mr($offset).read().bits() == 0 {
            if $self._periodic {
                reg.mcr().modify(|_, w| w.mr0i().set_bit());
            }
            return Poll::Ready(());
        }
        if $offset == 1 && reg.mr($offset).read().bits() == 0 {
            if $self._periodic {
                reg.mcr().modify(|_, w| w.mr1i().set_bit());
            }
            return Poll::Ready(());
        }
        if $offset == 2 && reg.mr($offset).read().bits() == 0 {
            if $self._periodic {
                reg.mcr().modify(|_, w| w.mr2i().set_bit());
            }
            return Poll::Ready(());
        }
        if $offset == 3 && reg.mr($offset).read().bits() == 0 {
            if $self._periodic {
                reg.mcr().modify(|_, w| w.mr3i().set_bit());
            }
            return Poll::Ready(());
        }
    };
}

macro_rules! impl_counting_timer_start {
    ($timer:ident, $self:ident, $cycles:ident, $timer_intr:ident) => {
        let reg = unsafe { $timer::steal() };
        let offset = $self._id % CHANNEL_PER_MODULE;

        $self._timeout = $cycles;

        let curr_time = reg.tc().read().bits();

        if curr_time as u64 + $cycles as u64 > u32::MAX as u64 {
            let leftover = (curr_time as u64 + $cycles as u64) - u32::MAX as u64;
            let cycles = leftover as u32;
            unsafe {
                reg.mr(offset).write(|w| w.match_().bits(cycles));
            }
        } else {
            unsafe {
                reg.mr(offset).write(|w| w.match_().bits(curr_time + $cycles));
            }
        }

        if reg.tcr().read().cen().bit_is_clear() {
            reg.tcr().write(|w| w.crst().set_bit());
            reg.tcr().write(|w| w.crst().clear_bit());
            reg.tcr().write(|w| w.cen().set_bit());
            unsafe {
                interrupt::$timer_intr.unpend();
                interrupt::$timer_intr.enable();
            }
        }
    };
}

macro_rules! impl_capture_timer_start {
    ($timer:ident, $self:ident, $timer_intr:ident) => {
        let reg = unsafe { $timer::steal() };

        if reg.tcr().read().cen().bit_is_clear() {
            reg.tcr().write(|w| w.crst().set_bit());
            reg.tcr().write(|w| w.crst().clear_bit());
            reg.tcr().write(|w| w.cen().set_bit());
            unsafe {
                interrupt::$timer_intr.unpend();
                interrupt::$timer_intr.enable();
            }
        }
    };
}

macro_rules! impl_capture_timer_wait {
    ($timer:ident,$offset:ident, $self:ident) => {
        let reg = unsafe { $timer::steal() };
        if $offset == 0 && reg.cr($offset).read().bits() != 0 {
            if ($self._periodic) {
                reg.ccr().write(|w| w.cap0i().set_bit());
            }
        } else if $offset == 1 && reg.cr($offset).read().bits() != 0 {
            if ($self._periodic) {
                reg.ccr().write(|w| w.cap1i().set_bit());
            }
        } else if $offset == 2 && reg.cr($offset).read().bits() != 0 {
            if ($self._periodic) {
                reg.ccr().write(|w| w.cap2i().set_bit());
            }
        } else if $offset == 3 && reg.cr($offset).read().bits() != 0 {
            if ($self._periodic) {
                reg.ccr().write(|w| w.cap3i().set_bit());
            }
        } else {
            return Poll::Pending;
        }
        ($self._cb)(reg.cr($offset).read().bits());
        return Poll::Ready(());
    };
}

impl<F: Fn(u32)> CaptureTimer<F> {
    fn new(id: usize, callback: F, edge: CaptureChEdge, periodic: bool) -> Self {
        CaptureTimer {
            _clk_freq: 16000000,
            _cb: callback,
            _timeout: 0,
            _edge: edge,
            _id: id,
            _periodic: periodic,
        }
    }
}

impl<F: Fn(u32)> Timer for CaptureTimer<F> {
    async fn wait(&self) {
        // Implementation of waiting for the interrupt
        poll_fn(|cx| {
            WAKERS[self._id].register(cx.waker());

            let idx = (self._id - COUNT_CHANNEL) / CHANNEL_PER_MODULE;
            let offset = (self._id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;

            match idx {
                0 => {
                    impl_capture_timer_wait!(Ctimer0, offset, self);
                }
                1 => {
                    impl_capture_timer_wait!(Ctimer1, offset, self);
                }
                2 => {
                    impl_capture_timer_wait!(Ctimer2, offset, self);
                }
                3 => {
                    impl_capture_timer_wait!(Ctimer3, offset, self);
                }
                4 => {
                    impl_capture_timer_wait!(Ctimer4, offset, self);
                }
                _ => {
                    panic!("Invalid timer instance");
                }
            }
        })
        .await;
    }
    fn start_count(&mut self, _dur: u32) {
        panic!("Counting not supported for capture timer");
    }

    fn start_capture(&self, event_input: u32) {
        // Just enable the interrupt for capture event
        let idx = (self._id - COUNT_CHANNEL) / CHANNEL_PER_MODULE;
        let offset = (self._id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;
        let reg = unsafe { Inputmux::steal() };

        reg.ct32bit_cap(idx)
            .ct32bit_cap_sel(offset)
            .modify(|_, w| unsafe { w.bits(event_input) });

        match idx {
            0 => {
                impl_capture_timer_start!(Ctimer0, self, CTIMER0);
            }
            1 => {
                impl_capture_timer_start!(Ctimer1, self, CTIMER1);
            }
            2 => {
                impl_capture_timer_start!(Ctimer2, self, CTIMER2);
            }
            3 => {
                impl_capture_timer_start!(Ctimer3, self, CTIMER3);
            }
            4 => {
                impl_capture_timer_start!(Ctimer4, self, CTIMER4);
            }
            _ => panic!("Invalid timer instance"),
        }
    }
}

impl<F: Fn()> CountingTimer<F> {
    fn new(id: usize, callback: F, periodic: bool) -> Self {
        CountingTimer {
            _id: id,
            _clk_freq: 16000000,
            _cb: callback,
            _timeout: 0,
            _periodic: periodic,
        }
    }
}

impl<F> Timer for CountingTimer<F>
where
    F: Fn(),
{
    fn start_count(&mut self, duration_us: u32) {
        //TODO: Start the timer
        //      - Program the match register
        //      - Enable the interrupt for the channel
        let idx = self._id / CHANNEL_PER_MODULE;
        let dur = (duration_us as u64 * self._clk_freq as u64) / 1000000;

        if dur > (u32::MAX) as u64 {
            panic!("Count value is too large");
        }

        let cycles = dur as u32;

        match idx {
            0 => {
                impl_counting_timer_start!(Ctimer0, self, cycles, CTIMER0);
            }
            1 => {
                impl_counting_timer_start!(Ctimer1, self, cycles, CTIMER1);
            }
            2 => {
                impl_counting_timer_start!(Ctimer2, self, cycles, CTIMER2);
            }
            3 => {
                impl_counting_timer_start!(Ctimer3, self, cycles, CTIMER3);
            }
            4 => {
                impl_counting_timer_start!(Ctimer4, self, cycles, CTIMER4);
            }
            _ => panic!("Invalid timer instance"),
        }
    }
    async fn wait(&self) {
        // Implementation of waiting for the interrupt
        poll_fn(|cx| {
            // Register the waker
            let idx = self._id / CHANNEL_PER_MODULE;
            let offset = self._id % CHANNEL_PER_MODULE;
            WAKERS[self._id].register(cx.waker());

            match idx {
                0 => {
                    impl_counting_timer_wait!(Ctimer0, offset, self);
                }
                1 => {
                    impl_counting_timer_wait!(Ctimer1, offset, self);
                }
                2 => {
                    impl_counting_timer_wait!(Ctimer2, offset, self);
                }
                3 => {
                    impl_counting_timer_wait!(Ctimer3, offset, self);
                }
                4 => {
                    impl_counting_timer_wait!(Ctimer4, offset, self);
                }
                _ => {
                    panic!("Invalid timer instance");
                }
            }

            Poll::Pending
        })
        .await;
        (self._cb)();
    }
    fn start_capture(&self, _event_input: u32) {
        panic!("Capture not supported for counting timer");
    }
}

/// Trait representing the state of a CTimerManager.
pub trait ModuleState {}
/// Represents an unallocated state for the CTimerManager.
pub struct Unallocated;
/// Represents an uninitialized state for the CTimerManager.
pub struct Uninitialized;
/// Represents the initialized state of the CTimerManager.
pub struct Initialized {
    ch_arr: [Channel; TOTAL_CHANNELS],
}

macro_rules! Impl_module_state {
	($($func:ident),+) => {
	    $(
	        impl ModuleState for $func {}
	     )+
	}
}

Impl_module_state!(Unallocated, Uninitialized, Initialized);

/// A manager for handling CTimer modules with different states.
pub struct CTimerManager<T: ModuleState> {
    state: T,
}

// We can use this for any shared behavior between all the states
impl<T: ModuleState> CTimerManager<T> {}

impl CTimerManager<Unallocated> {
    /// Creates a new `CTimerManager` in the uninitialized state.
    pub fn new() -> CTimerManager<Uninitialized> {
        CTimerManager { state: Uninitialized }
    }
}

impl CTimerManager<Uninitialized> {
    /// Initializes the timer modules and returns a `CTimerManager` in the initialized state.
    pub fn init_timer_modules(self) -> CTimerManager<Initialized> {
        let reg = unsafe { Clkctl1::steal() };

        // Initialization steps from NXP TRM
        //
        // • Enable the clock to the CTIMER in the CLKCTL1_PSCCTL2 register
        //          This enables the register interface and the peripheral function clock.
        reg.pscctl2_set().write(|w| w.ct32bit0_clk_set().set_clock());
        reg.pscctl2_set().write(|w| w.ct32bit1_clk_set().set_clock());
        reg.pscctl2_set().write(|w| w.ct32bit2_clk_set().set_clock());
        reg.pscctl2_set().write(|w| w.ct32bit3_clk_set().set_clock());
        reg.pscctl2_set().write(|w| w.ct32bit4_clk_set().set_clock());

        // • Enable the clock to the PIMCTL in the CLKCTL1_PSCCTL2 register
        reg.pscctl2_set().write(|w| w.pimctl_clk_set().set_bit());

        // • Select a clock source for the CTIMER using the appropriate CT32BIT0FCLKSEL
        // register (see Section 4.5.2.55 through Section 4.5.2.59).
        reg.ct32bitfclksel(0).write(|w| w.sel().sfro_clk());
        reg.ct32bitfclksel(1).write(|w| w.sel().sfro_clk());
        reg.ct32bitfclksel(2).write(|w| w.sel().sfro_clk());
        reg.ct32bitfclksel(3).write(|w| w.sel().sfro_clk());
        reg.ct32bitfclksel(4).write(|w| w.sel().sfro_clk());

        // • Clear the CTIMER peripheral reset in the RSTCTL1_PRSTCTL2 register
        // (Section 4.5.4.4) by writing to the RSTCTL1_PRSTCTL2_CLR register (Section 4.5.4.10).
        let reg = unsafe { Rstctl1::steal() };
        reg.prstctl2_clr().write(|w| w.ct32bit0_rst_clr().clr_reset());
        reg.prstctl2_clr().write(|w| w.ct32bit1_rst_clr().clr_reset());
        reg.prstctl2_clr().write(|w| w.ct32bit2_rst_clr().clr_reset());
        reg.prstctl2_clr().write(|w| w.ct32bit3_rst_clr().clr_reset());
        reg.prstctl2_clr().write(|w| w.ct32bit4_rst_clr().clr_reset());

        CTimerManager {
            state: Initialized {
                ch_arr: [Channel::new(); TOTAL_CHANNELS],
            },
        }
    }
}

impl CTimerManager<Initialized> {
    // Factory method for Abstract counting timer creation for user
    /// Requests a counting timer with the specified callback and periodicity.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to be called when the timer expires.
    /// * `periodic` - A boolean indicating whether the timer should be periodic.
    pub fn request_counting_timer(&mut self, callback: impl Fn(), periodic: bool) -> impl Timer {
        let id = self.allocate_channel(TimerType::Counting).unwrap_or(u32::MAX as usize);

        if id == u32::MAX as usize {
            panic!("No free channel available");
        }

        let timer_idx = id / CHANNEL_PER_MODULE;

        match timer_idx {
            0 => {
                impl_counting_timer_setup!(Ctimer0, id);
            }
            1 => {
                impl_counting_timer_setup!(Ctimer1, id);
            }
            2 => {
                impl_counting_timer_setup!(Ctimer2, id);
            }
            3 => {
                impl_counting_timer_setup!(Ctimer3, id);
            }
            4 => {
                impl_counting_timer_setup!(Ctimer4, id);
            }
            _ => {
                panic!("Invalid timer instance");
            }
        }
        CountingTimer::new(id, callback, periodic)
    }

    /// Requests a capture timer with the specified callback and edge.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to be called on capture event.
    /// * `edge` - The edge type for the capture channel.
    pub fn request_capture_timer(&mut self, callback: impl Fn(u32), edge: CaptureChEdge, periodic: bool) -> impl Timer {
        let id = self.allocate_channel(TimerType::Capture).unwrap_or(u32::MAX as usize);

        if id == u32::MAX as usize {
            panic!("No free channel available");
        }

        // map logical timer id to physical controller
        let timer_idx = (id - COUNT_CHANNEL) / CHANNEL_PER_MODULE;

        match timer_idx {
            0 => {
                impl_capture_timer_setup!(Ctimer0, edge, id);
            }
            1 => {
                impl_capture_timer_setup!(Ctimer1, edge, id);
            }
            2 => {
                impl_capture_timer_setup!(Ctimer2, edge, id);
            }
            3 => {
                impl_capture_timer_setup!(Ctimer3, edge, id);
            }
            4 => {
                impl_capture_timer_setup!(Ctimer4, edge, id);
            }
            _ => panic!("Invalid timer instance"),
        }

        CaptureTimer::new(id, callback, edge, periodic)
    }

    fn allocate_channel(&mut self, timertype: TimerType) -> Option<usize> {
        match timertype {
            TimerType::Counting => self.allocate_counting_channel(),
            TimerType::Capture => self.allocate_capture_channel(),
        }
    }

    fn allocate_counting_channel(&mut self) -> Option<usize> {
        for i in 0..COUNT_CHANNEL {
            if self.state.ch_arr[i].allocated == false {
                self.state.ch_arr[i].allocated = true;
                return Some(i);
            }
        }
        None
    }

    fn allocate_capture_channel(&mut self) -> Option<usize> {
        for i in COUNT_CHANNEL..TOTAL_CHANNELS {
            if self.state.ch_arr[i].allocated == false {
                self.state.ch_arr[i].allocated = true;
                return Some(i);
            }
        }
        return None;
    }
}

#[cfg(feature = "rt")]
fn irq_handler(inst: u32) {
    match inst {
        0 => {
            irq_handler_impl!(Ctimer0, 0, 1, 2, 3, 20, 21, 22, 23);
        }
        1 => {
            irq_handler_impl!(Ctimer1, 4, 5, 6, 7, 24, 25, 26, 27);
        }
        2 => {
            irq_handler_impl!(Ctimer2, 8, 9, 10, 11, 28, 29, 30, 31);
        }
        3 => {
            irq_handler_impl!(Ctimer3, 12, 13, 14, 15, 32, 33, 34, 35);
        }
        4 => {
            irq_handler_impl!(Ctimer4, 16, 17, 18, 19, 36, 37, 38, 39);
        }
        _ => {
            panic!("Invalid timer instance");
        }
    }
}

impl interrupt::typelevel::Handler<crate::interrupt::typelevel::CTIMER0> for CtimerInterruptHandler {
    unsafe fn on_interrupt() {
        irq_handler(0);
    }
}

impl interrupt::typelevel::Handler<crate::interrupt::typelevel::CTIMER1> for CtimerInterruptHandler {
    unsafe fn on_interrupt() {
        irq_handler(1);
    }
}

impl interrupt::typelevel::Handler<crate::interrupt::typelevel::CTIMER2> for CtimerInterruptHandler {
    unsafe fn on_interrupt() {
        irq_handler(2);
    }
}

impl interrupt::typelevel::Handler<crate::interrupt::typelevel::CTIMER3> for CtimerInterruptHandler {
    unsafe fn on_interrupt() {
        irq_handler(3);
    }
}

impl interrupt::typelevel::Handler<crate::interrupt::typelevel::CTIMER4> for CtimerInterruptHandler {
    unsafe fn on_interrupt() {
        irq_handler(4);
    }
}
