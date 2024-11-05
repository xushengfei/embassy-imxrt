//! Timer module for the NXP RT6xx family of microcontrollers
use core::future::poll_fn;
use core::task::Poll;

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use paste::paste;

use crate::iopctl::{DriveMode, DriveStrength, Inverter, IopctlPin as Pin, Pull, SlewRate};
use crate::pac::{Clkctl1, Inputmux, Rstctl1};
use crate::{interrupt, Peripheral};

const COUNT_CHANNEL: usize = 20;
const CAPTURE_CHANNEL: usize = 20;
const TOTAL_CHANNELS: usize = COUNT_CHANNEL + CAPTURE_CHANNEL;
const CHANNEL_PER_MODULE: usize = 4;
const TIMER_MODULES: usize = 5;
enum TimerModule {
    CTIMER0,
    CTIMER1,
    CTIMER2,
    CTIMER3,
    CTIMER4,
}
enum TimerChannelNum {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}
/// Enum representing the logical Count/capture channel output.
pub enum TriggerOutput {
    /// Match/Capture output trigger 0
    TrigOut0,
    /// Match output 1
    TrigOut1,
    /// Match output 2
    TrigOut2,
    /// Match output 3
    TrigOut3,
    /// Match output 4
    TrigOut4,
    /// Match output 5
    TrigOut5,
    /// Match output 6
    TrigOut6,
    /// Match output 7
    TrigOut7,
    /// Match output 8
    TrigOut8,
    /// Match output 9
    TrigOut9,
    /// Match output 10
    TrigOut10,
    /// Match output 11
    TrigOut11,
    /// Match output 12
    TrigOut12,
    /// Match output 13
    TrigOut13,
    /// Match output 14
    TrigOut14,
    /// Match output 15
    TrigOut15,
    /// Match output 16
    TrigOut16,
    /// Match output 17
    TrigOut17,
    /// Match output 18
    TrigOut18,
    /// Match output 19
    TrigOut19,
    /// Match output 20
    TrigOut20,
    /// Match output 21
    TrigOut21,
    /// Match output 22
    TrigOut22,
    /// Match output 23
    TrigOut23,
    /// Match output 24
    TrigOut24,
}

/// Enum representing the logical capture channel input.
pub enum TriggerInput {
    /// Capture input 0
    TrigIn0,
    /// Capture input 1
    TrigIn1,
    /// Capture input 2
    TrigIn2,
    /// Capture input 3
    TrigIn3,
    /// Capture input 4
    TrigIn4,
    /// Capture input 5
    TrigIn5,
    /// Capture input 6
    TrigIn6,
    /// Capture input 7
    TrigIn7,
    /// Capture input 8
    TrigIn8,
    /// Capture input 9
    TrigIn9,
    /// Capture input 10
    TrigIn10,
    /// Capture input 11
    TrigIn11,
    /// Capture input 12
    TrigIn12,
    /// Capture input 13
    TrigIn13,
    /// Capture input 14
    TrigIn14,
    /// Capture input 15
    TrigIn15,
    /// Capture input 16
    TrigIn16,
    /// Capture input 17
    TrigIn17,
    /// Capture input 18
    TrigIn18,
    /// Capture input 19
    TrigIn19,
    /// Capture input 20
    TrigIn20,
    /// Capture input 21
    TrigIn21,
    /// Capture input 22
    TrigIn22,
    /// Capture input 23
    TrigIn23,
    /// Capture input 24
    TrigIn24,
}

const TIMER_MODULES_ARR: [TimerModule; TIMER_MODULES] = [
    TimerModule::CTIMER0,
    TimerModule::CTIMER1,
    TimerModule::CTIMER2,
    TimerModule::CTIMER3,
    TimerModule::CTIMER4,
];

const TIMER_CHANNELS_ARR: [TimerChannelNum; CHANNEL_PER_MODULE] = [
    TimerChannelNum::Channel0,
    TimerChannelNum::Channel1,
    TimerChannelNum::Channel2,
    TimerChannelNum::Channel3,
];

static WAKERS: [AtomicWaker; TOTAL_CHANNELS] = [const { AtomicWaker::new() }; TOTAL_CHANNELS];

pub use embedded_hal_02::timer::{Cancel, CountDown, Periodic};

/// Time type for the timer module
#[derive(PartialEq)]
pub enum TimerType {
    /// Counting timer
    Counting,
    /// Capture timer
    Capture,
}

#[derive(PartialEq, Clone, Copy)]
/// Enum representing the edge type for capture channels.
pub enum CaptureChEdge {
    /// Rising edge
    Rising,
    /// Falling edge
    Falling,
    /// Both edges
    Both,
}

mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}

/// A timer that captures events based on a specified edge and calls a user-defined callback.
pub struct CaptureTimer<M: Mode> {
    id: usize,
    _clk_freq: u32,
    _timeout: u32,
    _edge: CaptureChEdge,
    periodic: bool,
    hist: u32,
    timer_type: TimerType,
    _phantom: core::marker::PhantomData<M>,
    info: Info,
}

/// A timer that counts down to zero and calls a user-defined callback.
pub struct CountingTimer<M: Mode> {
    id: usize,
    clk_freq: u32,
    timeout: u32,
    periodic: bool,
    timer_type: TimerType,
    _phantom: core::marker::PhantomData<M>,
    info: Info,
}

struct Info {
    regs: &'static crate::pac::ctimer0::RegisterBlock,
    index: usize,
    ch_idx: usize,
}

trait SealedInstance {
    fn info() -> Info;
}

/// SPI instance trait.
/// shared functions between Controller and Target operation
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + 'static + Send {
    /// Interrupt for this SPI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

/// Interrupt handler for the CTimer modules.
pub struct CtimerInterruptHandler<T: Instance> {
    _phantom: core::marker::PhantomData<T>,
}

impl Info {
    fn cap_timer_intr_enable(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.ccr().modify(|_, w| w.cap0i().set_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.ccr().modify(|_, w| w.cap1i().set_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.ccr().modify(|_, w| w.cap2i().set_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.ccr().modify(|_, w| w.cap3i().set_bit());
            }
        }
    }
    fn cap_timer_intr_disable(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.ccr().modify(|_, w| w.cap0i().clear_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.ccr().modify(|_, w| w.cap1i().clear_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.ccr().modify(|_, w| w.cap2i().clear_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.ccr().modify(|_, w| w.cap3i().clear_bit());
            }
        }
    }
    fn cap_timer_enable_rising_edge_event(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.ccr().modify(|_, w| w.cap0re().set_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.ccr().modify(|_, w| w.cap1re().set_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.ccr().modify(|_, w| w.cap2re().set_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.ccr().modify(|_, w| w.cap3re().set_bit());
            }
        }
    }
    fn cap_timer_enable_falling_edge_event(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.ccr().modify(|_, w| w.cap0fe().set_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.ccr().modify(|_, w| w.cap1fe().set_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.ccr().modify(|_, w| w.cap2fe().set_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.ccr().modify(|_, w| w.cap3fe().set_bit());
            }
        }
    }
    fn cap_timer_disable_rising_edge_event(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.ccr().modify(|_, w| w.cap0re().clear_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.ccr().modify(|_, w| w.cap1re().clear_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.ccr().modify(|_, w| w.cap2re().clear_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.ccr().modify(|_, w| w.cap3re().clear_bit());
            }
        }
    }
    fn cap_timer_disable_falling_edge_event(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.ccr().modify(|_, w| w.cap0fe().clear_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.ccr().modify(|_, w| w.cap1fe().clear_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.ccr().modify(|_, w| w.cap2fe().clear_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.ccr().modify(|_, w| w.cap3fe().clear_bit());
            }
        }
    }
    fn count_timer_enable_interrupt(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.mcr().modify(|_, w| w.mr0i().set_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.mcr().modify(|_, w| w.mr1i().set_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.mcr().modify(|_, w| w.mr2i().set_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.mcr().modify(|_, w| w.mr3i().set_bit());
            }
        }
    }
    fn count_timer_disable_interrupt(&self) {
        let reg = self.regs;
        let channel = self.ch_idx;
        match TIMER_CHANNELS_ARR[channel] {
            TimerChannelNum::Channel0 => {
                reg.mcr().modify(|_, w| w.mr0i().clear_bit());
            }
            TimerChannelNum::Channel1 => {
                reg.mcr().modify(|_, w| w.mr1i().clear_bit());
            }
            TimerChannelNum::Channel2 => {
                reg.mcr().modify(|_, w| w.mr2i().clear_bit());
            }
            TimerChannelNum::Channel3 => {
                reg.mcr().modify(|_, w| w.mr3i().clear_bit());
            }
        }
    }
}

macro_rules! impl_instance {
    ($n:expr, $channel:expr) => {
        paste! {
            impl SealedInstance for crate::peripherals::[<CTIMER $n _ COUNT _ CHANNEL $channel>] {
                fn info() -> Info {
                    Info {
                        regs: unsafe { &*crate::pac::[<Ctimer $n>]::ptr() },
                        index: $n,
                        ch_idx: $channel,
                    }
                }
            }
            impl SealedInstance for crate::peripherals::[<CTIMER $n _ CAPTURE _ CHANNEL $channel>] {
                fn info() -> Info {
                    Info {
                        regs: unsafe { &*crate::pac::[<Ctimer $n>]::ptr() },
                        index: $n,
                        ch_idx: $channel,
                    }
                }
            }
            impl Instance for crate::peripherals::[<CTIMER $n _ COUNT _ CHANNEL $channel>] {
                type Interrupt = crate::interrupt::typelevel::[<CTIMER $n>];
            }
            impl Instance for crate::peripherals::[<CTIMER $n _ CAPTURE _ CHANNEL $channel>] {
                type Interrupt = crate::interrupt::typelevel::[<CTIMER $n>];
            }
        }
    };
}

impl_instance!(0, 0); // CTIMER0 Channel 0
impl_instance!(0, 1); // CTIMER0 Channel 1
impl_instance!(0, 2); // CTIMER0 Channel 2
impl_instance!(0, 3); // CTIMER0 Channel 3

impl_instance!(1, 0); // CTIMER1 Channel 0
impl_instance!(1, 1); // CTIMER1 Channel 1
impl_instance!(1, 2); // CTIMER1 Channel 2
impl_instance!(1, 3); // CTIMER1 Channel 3

impl_instance!(2, 0); // CTIMER2 Channel 0
impl_instance!(2, 1); // CTIMER2 Channel 1
impl_instance!(2, 2); // CTIMER2 Channel 2
impl_instance!(2, 3); // CTIMER2 Channel 3

impl_instance!(3, 0); // CTIMER3 Channel 0
impl_instance!(3, 1); // CTIMER3 Channel 1
impl_instance!(3, 2); // CTIMER3 Channel 2
impl_instance!(3, 3); // CTIMER3 Channel 3

impl_instance!(4, 0); // CTIMER4 Channel 0
impl_instance!(4, 1); // CTIMER4 Channel 1
impl_instance!(4, 2); // CTIMER4 Channel 2
impl_instance!(4, 3); // CTIMER4 Channel 3

impl<M: Mode> CaptureTimer<M> {
    /// Start the capture timer
    pub fn start(
        &mut self,
        count: u32,
        event_input: Option<TriggerInput>,
        event_output: Option<TriggerOutput>,
        event_pin: impl CaptureEvent,
    ) {
        let module = self.info.index;
        let channel = self.info.ch_idx;

        // SAFETY: This has no safety impact as we are getting a singleton register instance here and its dropped it the end of the function
        let reg = unsafe { Inputmux::steal() };

        if self.timer_type == TimerType::Capture && (event_output.is_some() || count != 0) {
            panic!("No output event for capture timer");
        }
        event_pin.capture_event();
        reg.ct32bit_cap(module)
            .ct32bit_cap_sel(channel)
            .modify(|_, w| match event_input {
                Some(TriggerInput::TrigIn0) => w.capn_sel().ct_inp0(),
                Some(TriggerInput::TrigIn1) => w.capn_sel().ct_inp1(),
                Some(TriggerInput::TrigIn2) => w.capn_sel().ct_inp2(),
                Some(TriggerInput::TrigIn3) => w.capn_sel().ct_inp3(),
                Some(TriggerInput::TrigIn4) => w.capn_sel().ct_inp4(),
                Some(TriggerInput::TrigIn5) => w.capn_sel().ct_inp5(),
                Some(TriggerInput::TrigIn6) => w.capn_sel().ct_inp6(),
                Some(TriggerInput::TrigIn7) => w.capn_sel().ct_inp7(),
                Some(TriggerInput::TrigIn8) => w.capn_sel().ct_inp8(),
                Some(TriggerInput::TrigIn9) => w.capn_sel().ct_inp9(),
                Some(TriggerInput::TrigIn10) => w.capn_sel().ct_inp10(),
                Some(TriggerInput::TrigIn11) => w.capn_sel().ct_inp11(),
                Some(TriggerInput::TrigIn12) => w.capn_sel().ct_inp12(),
                Some(TriggerInput::TrigIn13) => w.capn_sel().ct_inp13(),
                Some(TriggerInput::TrigIn14) => w.capn_sel().ct_inp14(),
                Some(TriggerInput::TrigIn15) => w.capn_sel().ct_inp15(),
                Some(TriggerInput::TrigIn16) => w.capn_sel().shared_i2s0_ws(),
                Some(TriggerInput::TrigIn17) => w.capn_sel().shared_i2s1_ws(),
                Some(TriggerInput::TrigIn18) => w.capn_sel().usb1_frame_toggle(),
                Some(_) => panic!("Invalid input event for capture timer"),
                None => {
                    panic!("No input event for capture timer");
                }
            });

        let reg = self.info.regs;

        if reg.tcr().read().cen().bit_is_clear() {
            reg.tcr().write(|w| w.crst().set_bit());
            reg.tcr().write(|w| w.crst().clear_bit());
            reg.tcr().write(|w| w.cen().set_bit());
            unsafe {
                // SAFETY: No safety impact as we are enabling the interrupt for the module
                match TIMER_MODULES_ARR[module] {
                    TimerModule::CTIMER0 => {
                        interrupt::CTIMER0.unpend();
                        interrupt::CTIMER0.enable();
                    }
                    TimerModule::CTIMER1 => {
                        interrupt::CTIMER1.unpend();
                        interrupt::CTIMER1.enable();
                    }
                    TimerModule::CTIMER2 => {
                        interrupt::CTIMER2.unpend();
                        interrupt::CTIMER2.enable();
                    }
                    TimerModule::CTIMER3 => {
                        interrupt::CTIMER3.unpend();
                        interrupt::CTIMER3.enable();
                    }
                    TimerModule::CTIMER4 => {
                        interrupt::CTIMER4.unpend();
                        interrupt::CTIMER4.enable();
                    }
                }
            }
        }
    }
    fn capture_timer_setup(edge: CaptureChEdge, info: &Info) {
        info.cap_timer_intr_enable();
        match edge {
            CaptureChEdge::Rising => {
                info.cap_timer_enable_rising_edge_event();
            }
            CaptureChEdge::Falling => {
                info.cap_timer_enable_falling_edge_event();
            }
            CaptureChEdge::Both => {
                panic!("Both edge not supported yet");
            }
        }
    }
}

impl CaptureTimer<Async> {
    /// Creates a new `CaptureTimer` in asynchronous mode.
    pub fn new_async<T: Instance>(_inst: T, edge: CaptureChEdge, periodic: bool) -> Self {
        let info = T::info();
        let module = info.index;
        Self::capture_timer_setup(edge, &info);
        Self {
            id: COUNT_CHANNEL + module * CHANNEL_PER_MODULE + info.ch_idx,
            _clk_freq: 16000000,
            _timeout: 0,
            _edge: edge,
            periodic,
            hist: 0,
            timer_type: TimerType::Capture,
            _phantom: core::marker::PhantomData,
            info,
        }
    }

    /// Waits asynchronously for the capture timer to record an event timestamp.
    pub async fn wait(&mut self) {
        // Implementation of waiting for the interrupt
        poll_fn(|cx| {
            let reg = self.info.regs;
            WAKERS[self.id].register(cx.waker());

            if reg.cr(self.info.ch_idx).read().bits() != self.hist {
                self.hist = reg.cr(self.info.ch_idx).read().bits();
                if self.periodic {
                    self.info.cap_timer_intr_enable();
                }
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

impl CaptureTimer<Blocking> {
    /// Creates a new `CaptureTimer` in blocking mode.
    pub fn new_blocking<T: Instance>(_inst: T, edge: CaptureChEdge, periodic: bool) -> Self {
        let info = T::info();
        let module = info.index;
        Self::capture_timer_setup(edge, &info);
        Self {
            id: COUNT_CHANNEL + module * CHANNEL_PER_MODULE + info.ch_idx,
            _clk_freq: 16000000,
            _timeout: 0,
            _edge: edge,
            periodic,
            hist: 0,
            timer_type: TimerType::Capture,
            _phantom: core::marker::PhantomData,
            info,
        }
    }
    /// Waits synchronously for the capture timer
    pub fn wait(&mut self) {
        let reg = self.info.regs;

        loop {
            if reg.cr(self.info.ch_idx).read().bits() != self.hist {
                self.hist = reg.cr(self.info.ch_idx).read().bits();
                if self.periodic {
                    self.info.cap_timer_intr_enable();
                }
                break;
            }
        }
    }
}

impl<M: Mode> CountingTimer<M> {
    /// Starts a new `CountingTimer`
    pub fn start(&mut self, count: u32, event_input: Option<TriggerInput>, event_output: Option<TriggerOutput>) {
        let module = self.info.index;
        let dur = (count as u64 * self.clk_freq as u64) / 1000000;
        let cycles = dur as u32;
        let reg = self.info.regs;
        let offset = self.info.ch_idx;
        let curr_time = reg.tc().read().bits();

        if self.timer_type == TimerType::Capture && event_input.is_some() {
            panic!("No output event for capture timer");
        }
        if dur > (u32::MAX) as u64 {
            panic!("Count value is too large");
        }
        if event_output.is_some() {
            panic!("No output event for match support");
        }

        self.timeout = cycles;

        if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
            let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
            let cycles = leftover as u32;
            unsafe {
                // SAFETY: It has no safety impact as we are writing new value to match register here
                reg.mr(offset).write(|w| w.match_().bits(cycles));
            }
        } else {
            unsafe {
                //SAFETY: It has no safety impact as we are writing new value to match register here
                reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
            }
        }

        if reg.tcr().read().cen().bit_is_clear() {
            reg.tcr().write(|w| w.crst().set_bit());
            reg.tcr().write(|w| w.crst().clear_bit());
            reg.tcr().write(|w| w.cen().set_bit());
            unsafe {
                // SAFETY: No safety impact as we are enabling the interrupt for the module
                match TIMER_MODULES_ARR[module] {
                    TimerModule::CTIMER0 => {
                        interrupt::CTIMER0.unpend();
                        interrupt::CTIMER0.enable();
                    }
                    TimerModule::CTIMER1 => {
                        interrupt::CTIMER1.unpend();
                        interrupt::CTIMER1.enable();
                    }
                    TimerModule::CTIMER2 => {
                        interrupt::CTIMER2.unpend();
                        interrupt::CTIMER2.enable();
                    }
                    TimerModule::CTIMER3 => {
                        interrupt::CTIMER3.unpend();
                        interrupt::CTIMER3.enable();
                    }
                    TimerModule::CTIMER4 => {
                        interrupt::CTIMER4.unpend();
                        interrupt::CTIMER4.enable();
                    }
                }
            }
        }
    }
}

impl CountingTimer<Async> {
    /// Creates a new `CountingTimer` in asynchronous mode.
    pub fn new_async<T: Instance>(_inst: T, periodic: bool, timertype: TimerType) -> Self {
        let info = T::info();
        info.count_timer_enable_interrupt();
        Self {
            id: info.index * CHANNEL_PER_MODULE + info.ch_idx,
            clk_freq: 16000000,
            timeout: 0,
            periodic,
            timer_type: timertype,
            _phantom: core::marker::PhantomData,
            info,
        }
    }
    /// Waits asynchronously for the countdown timer to complete.
    pub async fn wait(&mut self) {
        // Implementation of waiting for the interrupt
        poll_fn(|cx| {
            // Register the waker
            let channel = self.info.ch_idx;
            let reg = self.info.regs;
            WAKERS[self.id].register(cx.waker());

            if reg.mr(channel).read().bits() == 0 {
                if self.periodic {
                    let cycles = self.timeout;
                    let curr_time = reg.tc().read().bits();

                    if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                        let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                        let cycles = leftover as u32;
                        unsafe {
                            //SAFETY: It has no safety impact as we are writing new value to match register here
                            reg.mr(channel).write(|w| w.match_().bits(cycles));
                        }
                    } else {
                        unsafe {
                            //SAFETY: It has no safety impact as we are writing new value to match register here
                            reg.mr(channel).write(|w| w.match_().bits(curr_time + cycles));
                        }
                    }
                    self.info.count_timer_enable_interrupt();
                }
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;
    }
}

impl CountingTimer<Blocking> {
    /// Creates a new `CountingTimer` in blocking mode.
    pub fn new_blocking<T: Instance>(_inst: T, periodic: bool, timertype: TimerType) -> Self {
        let info = T::info();
        info.count_timer_enable_interrupt();
        Self {
            id: info.index * CHANNEL_PER_MODULE + info.ch_idx,
            clk_freq: 16000000,
            timeout: 0,
            periodic,
            timer_type: timertype,
            _phantom: core::marker::PhantomData,
            info,
        }
    }

    /// Waits synchronously for the countdown timer to complete.
    pub fn wait(&mut self) {
        let channel = self.info.ch_idx;
        let reg = self.info.regs;

        loop {
            if reg.mr(channel).read().bits() == 0 {
                if self.periodic {
                    let cycles = self.timeout;
                    let curr_time = reg.tc().read().bits();

                    if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                        let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                        let cycles = leftover as u32;
                        unsafe {
                            //SAFETY: It has no safety impact as we are writing new value to match register here
                            reg.mr(channel).write(|w| w.match_().bits(cycles));
                        }
                    } else {
                        unsafe {
                            //SAFETY: It has no safety impact as we are writing new value to match register here
                            reg.mr(channel).write(|w| w.match_().bits(curr_time + cycles));
                        }
                    }
                    self.info.count_timer_enable_interrupt();
                }
                break;
            }
        }
    }
}

impl<M: Mode> Drop for CountingTimer<M> {
    fn drop(&mut self) {
        self.info.count_timer_disable_interrupt();
        self.info.regs.mr(self.info.ch_idx).write(|w| unsafe {
            // SAFETY: It has no safety impact as we are clearing match register here
            w.match_().bits(0)
        });
    }
}

impl<M: Mode> Drop for CaptureTimer<M> {
    fn drop(&mut self) {
        info!(
            "Dropping CaptureTimer for module {} channel {}",
            self.info.index, self.info.ch_idx
        );
        self.info.cap_timer_intr_disable();
        self.info.cap_timer_disable_falling_edge_event();
        self.info.cap_timer_disable_rising_edge_event();
    }
}

/// Initializes the timer modules and returns a `CTimerManager` in the initialized state.
pub fn init_timer_modules() {
    // SAFETY: This has no safety impact as we are getting a singleton register instance here and its dropped it the end of the function
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

    // SAFETY: This has no safety impact as we are getting a singleton register instance here and its dropped it the end of the function
    let reg = unsafe { Rstctl1::steal() };
    reg.prstctl2_clr().write(|w| w.ct32bit0_rst_clr().clr_reset());
    reg.prstctl2_clr().write(|w| w.ct32bit1_rst_clr().clr_reset());
    reg.prstctl2_clr().write(|w| w.ct32bit2_rst_clr().clr_reset());
    reg.prstctl2_clr().write(|w| w.ct32bit3_rst_clr().clr_reset());
    reg.prstctl2_clr().write(|w| w.ct32bit4_rst_clr().clr_reset());
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for CtimerInterruptHandler<T> {
    unsafe fn on_interrupt() {
        let module = T::info().index;
        let reg = T::info().regs;

        let ir = reg.ir().read();

        if ir.mr0int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr0i().clear_bit());
            reg.ir().modify(|_, w| w.mr0int().set_bit());
            reg.mr(0).write(|w| unsafe {
                // SAFETY: It has no safety impact as we are clearing match register here
                w.match_().bits(0)
            });
            WAKERS[module * CHANNEL_PER_MODULE].wake();
        }
        if ir.mr1int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr1i().clear_bit());
            reg.ir().modify(|_, w| w.mr1int().set_bit());
            reg.mr(1).write(|w| unsafe {
                // SAFETY: It has no safety impact as we are clearing match register here
                w.match_().bits(0)
            });
            WAKERS[module * CHANNEL_PER_MODULE + 1].wake();
        }
        if ir.mr2int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr2i().clear_bit());
            reg.ir().modify(|_, w| w.mr2int().set_bit());
            reg.mr(2).write(|w| unsafe {
                // SAFETY: It has no safety impact as we are clearing match register here
                w.match_().bits(0)
            });
            WAKERS[module * CHANNEL_PER_MODULE + 2].wake();
        }
        if ir.mr3int().bit_is_set() {
            reg.mcr().modify(|_, w| w.mr3i().clear_bit());
            reg.ir().modify(|_, w| w.mr3int().set_bit());
            reg.mr(3).write(|w| unsafe {
                // SAFETY: It has no safety impact as we are clearing match register here
                w.match_().bits(0)
            });
            WAKERS[module * CHANNEL_PER_MODULE + 3].wake();
        }
        if ir.cr0int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap0i().clear_bit());
            reg.ir().modify(|_, w| w.cr0int().set_bit());
            WAKERS[module * CHANNEL_PER_MODULE + COUNT_CHANNEL].wake();
        }
        if ir.cr1int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap1i().clear_bit());
            reg.ir().modify(|_, w| w.cr1int().set_bit());
            WAKERS[module * CHANNEL_PER_MODULE + COUNT_CHANNEL + 1].wake();
        }
        if ir.cr2int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap2i().clear_bit());
            reg.ir().modify(|_, w| w.cr2int().set_bit());
            WAKERS[module * CHANNEL_PER_MODULE + COUNT_CHANNEL + 2].wake();
        }
        if ir.cr3int().bit_is_set() {
            reg.ccr().modify(|_, w| w.cap3i().clear_bit());
            reg.ir().modify(|_, w| w.cr3int().set_bit());
            WAKERS[module * CHANNEL_PER_MODULE + COUNT_CHANNEL + 3].wake();
        }
    }
}

/// A trait for pins that can be used as capture event inputs.
pub trait CaptureEvent: Pin + crate::Peripheral {
    /// Configures the pin as a capture event input.
    fn capture_event(&self);
}
macro_rules! impl_pin {
    ($piom_n:ident, $fn:ident, $invert:ident) => {
        impl CaptureEvent for crate::peripherals::$piom_n {
            fn capture_event(&self) {
                self.set_function(crate::iopctl::Function::$fn);
                self.set_drive_mode(DriveMode::PushPull);
                self.set_pull(Pull::None);
                self.set_slew_rate(SlewRate::Slow);
                self.set_drive_strength(DriveStrength::Normal);
                self.disable_analog_multiplex();
                self.enable_input_buffer();
                self.set_input_inverter(Inverter::$invert);
            }
        }
    };
}

impl_pin!(PIO1_7, F4, Enabled);
