use crate::interrupt;
use crate::pac::Clkctl1;
use crate::pac::Rstctl1;
use crate::pac::{Ctimer0, Ctimer1, Ctimer2, Ctimer3, Ctimer4, Inputmux};
use core::future::poll_fn;
use core::task::Poll;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
//use void::Void;

static WAKERS: [AtomicWaker; TOTAL_CHANNELS] = [const { AtomicWaker::new() }; TOTAL_CHANNELS];

pub use embedded_hal_02::timer::{Cancel, CountDown, Periodic};

/////// Enums ///////////////////////
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

const COUNT_CHANNEL: usize = 20;
const CAPTURE_CHANNEL: usize = 20;
const TOTAL_CHANNELS: usize = COUNT_CHANNEL + CAPTURE_CHANNEL;
const CHANNEL_PER_MODULE: usize = 4;

// mod private {
//     pub trait Sealed {}
// }

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
            if idx == 0 {
                let reg = unsafe { Ctimer0::steal() };
                let mut data = reg.ccr().read().bits();
                if offset == 0 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x4;
                } else if offset == 1 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x20;
                } else if offset == 2 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x100;
                } else if offset == 3 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x800;
                } else {
                    return Poll::Pending;
                }
                reg.ccr().write(|w| unsafe { w.bits(data) });
                (self._cb)(reg.cr(offset).read().bits());
                return Poll::Ready(());
            }
            if idx == 1 {
                let reg = unsafe { Ctimer1::steal() };
                let mut data = reg.ccr().read().bits();
                if offset == 0 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x4;
                } else if offset == 1 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x20;
                } else if offset == 2 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x100;
                } else if offset == 3 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x800;
                } else {
                    return Poll::Pending;
                }
                reg.ccr().write(|w| unsafe { w.bits(data) });
                (self._cb)(reg.cr(offset).read().bits());
                return Poll::Ready(());
            }
            if idx == 2 {
                let reg = unsafe { Ctimer2::steal() };
                let mut data = reg.ccr().read().bits();
                if offset == 0 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x4;
                } else if offset == 1 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x20;
                } else if offset == 2 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x100;
                } else if offset == 3 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x800;
                } else {
                    return Poll::Pending;
                }
                reg.ccr().write(|w| unsafe { w.bits(data) });
                (self._cb)(reg.cr(offset).read().bits());
                return Poll::Ready(());
            }
            if idx == 3 {
                let reg = unsafe { Ctimer3::steal() };
                let mut data = reg.ccr().read().bits();
                if offset == 0 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x4;
                } else if offset == 1 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x20;
                } else if offset == 2 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x100;
                } else if offset == 3 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x800;
                } else {
                    return Poll::Pending;
                }
                reg.ccr().write(|w| unsafe { w.bits(data) });
                (self._cb)(reg.cr(offset).read().bits());
                return Poll::Ready(());
            }
            if idx == 4 {
                let reg = unsafe { Ctimer4::steal() };
                let mut data = reg.ccr().read().bits();
                if offset == 0 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x4;
                } else if offset == 1 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x20;
                } else if offset == 2 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x100;
                } else if offset == 3 && reg.cr(offset).read().bits() != 0 && self._periodic {
                    data |= 0x800;
                } else {
                    return Poll::Pending;
                }
                reg.ccr().write(|w| unsafe { w.bits(data) });
                (self._cb)(reg.cr(offset).read().bits());
                return Poll::Ready(());
            }
            Poll::Pending
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
            .write(|w| unsafe { w.bits(event_input) });

        if idx == 0 {
            let reg = unsafe { Ctimer0::steal() };

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER0.unpend();
                    interrupt::CTIMER0.enable();
                }
            }
        }
        if idx == 1 {
            let reg = unsafe { Ctimer1::steal() };

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER1.unpend();
                    interrupt::CTIMER1.enable();
                }
            }
        }
        if idx == 2 {
            let reg = unsafe { Ctimer2::steal() };

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER2.unpend();
                    interrupt::CTIMER2.enable();
                }
            }
        }
        if idx == 3 {
            let reg = unsafe { Ctimer3::steal() };

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER3.unpend();
                    interrupt::CTIMER3.enable();
                }
            }
        }
        if idx == 4 {
            let reg = unsafe { Ctimer4::steal() };

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER4.unpend();
                    interrupt::CTIMER4.enable();
                }
            }
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

        if idx == 0 {
            let offset = self._id % CHANNEL_PER_MODULE;
            let reg = unsafe { Ctimer0::steal() };

            self._timeout = cycles;

            let curr_time = reg.tc().read().bits();

            if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                let cycles = leftover as u32;
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(cycles));
                }
            } else {
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                }
            }

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER0.unpend();
                    interrupt::CTIMER0.enable();
                }
            }
        }
        if idx == 1 {
            let offset = self._id % CHANNEL_PER_MODULE;
            let reg = unsafe { Ctimer1::steal() };

            self._timeout = cycles;

            let curr_time = reg.tc().read().bits();

            if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                let cycles = leftover as u32;
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(cycles));
                }
            } else {
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                }
            }

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER1.unpend();
                    interrupt::CTIMER1.enable();
                }
            }
        }
        if idx == 2 {
            let offset = self._id % CHANNEL_PER_MODULE;
            let reg = unsafe { Ctimer2::steal() };

            self._timeout = cycles;

            let curr_time = reg.tc().read().bits();

            if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                let cycles = leftover as u32;
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(cycles));
                }
            } else {
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                }
            }

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER2.unpend();
                    interrupt::CTIMER2.enable();
                }
            }
        }
        if idx == 3 {
            let offset = self._id % CHANNEL_PER_MODULE;
            let reg = unsafe { Ctimer3::steal() };

            self._timeout = cycles;

            let curr_time = reg.tc().read().bits();

            if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                let cycles = leftover as u32;
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(cycles));
                }
            } else {
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                }
            }

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER3.unpend();
                    interrupt::CTIMER3.enable();
                }
            }
        }
        if idx == 4 {
            let offset = self._id % CHANNEL_PER_MODULE;
            let reg = unsafe { Ctimer4::steal() };

            self._timeout = cycles;

            let curr_time = reg.tc().read().bits();

            if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                let cycles = leftover as u32;
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(cycles));
                }
            } else {
                unsafe {
                    reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                }
            }

            if reg.tcr().read().cen().bit_is_clear() {
                reg.tcr().write(|w| w.crst().set_bit());
                reg.tcr().write(|w| w.crst().clear_bit());
                reg.tcr().write(|w| w.cen().set_bit());
                unsafe {
                    interrupt::CTIMER4.unpend();
                    interrupt::CTIMER4.enable();
                }
            }
        }
    }
    async fn wait(&self) {
        // Implementation of waiting for the interrupt
        poll_fn(|cx| {
            // Register the waker
            let idx = self._id / CHANNEL_PER_MODULE;
            let offset = self._id % CHANNEL_PER_MODULE;
            WAKERS[self._id].register(cx.waker());

            if idx == 0 {
                let reg = unsafe { Ctimer0::steal() };
                // Checking whether MR[Channel] is zero is based on the following logic-
                // For countdown timer, it makes sense to assume that MR is set to 1 initial value
                // and it is going to be counting down to 0.
                // With this logic, we can confirm whether this is the timer which expired.
                if self._periodic && reg.mr(offset).read().bits() == 0 {
                    let cycles = self._timeout;
                    let curr_time = reg.tc().read().bits();

                    if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                        let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                        let cycles = leftover as u32;
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(cycles));
                        }
                    } else {
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                        }
                    }
                }
                if offset == 0 && reg.mr(0).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x1;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 1 && reg.mr(1).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x8;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 2 && reg.mr(2).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x40;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 3 && reg.mr(3).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x200;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
            }
            if idx == 1 {
                let reg = unsafe { Ctimer1::steal() };
                if self._periodic && reg.mr(offset).read().bits() == 0 {
                    let cycles = self._timeout;
                    let curr_time = reg.tc().read().bits();

                    if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                        let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                        let cycles = leftover as u32;
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(cycles));
                        }
                    } else {
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                        }
                    }
                }
                if offset == 0 && reg.mr(0).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x1;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 1 && reg.mr(1).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x8;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 2 && reg.mr(2).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x40;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 3 && reg.mr(3).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x200;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
            }
            if idx == 2 {
                let reg = unsafe { Ctimer2::steal() };
                if self._periodic && reg.mr(offset).read().bits() == 0 {
                    let cycles = self._timeout;
                    let curr_time = reg.tc().read().bits();

                    if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                        let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                        let cycles = leftover as u32;
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(cycles));
                        }
                    } else {
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                        }
                    }
                }
                if offset == 0 && reg.mr(0).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x1;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 1 && reg.mr(1).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x8;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 2 && reg.mr(2).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x40;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 3 && reg.mr(3).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x200;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
            }
            if idx == 3 {
                let reg = unsafe { Ctimer3::steal() };
                if self._periodic && reg.mr(offset).read().bits() == 0 {
                    let cycles = self._timeout;
                    let curr_time = reg.tc().read().bits();

                    if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                        let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                        let cycles = leftover as u32;
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(cycles));
                        }
                    } else {
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                        }
                    }
                }
                if offset == 0 && reg.mr(0).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x1;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 1 && reg.mr(1).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x8;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 2 && reg.mr(2).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x40;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 3 && reg.mr(3).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x200;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
            }
            if idx == 4 {
                let reg = unsafe { Ctimer4::steal() };
                if self._periodic && reg.mr(offset).read().bits() == 0 {
                    let cycles = self._timeout;
                    let curr_time = reg.tc().read().bits();

                    if curr_time as u64 + cycles as u64 > u32::MAX as u64 {
                        let leftover = (curr_time as u64 + cycles as u64) - u32::MAX as u64;
                        let cycles = leftover as u32;
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(cycles));
                        }
                    } else {
                        unsafe {
                            reg.mr(offset).write(|w| w.match_().bits(curr_time + cycles));
                        }
                    }
                }
                if offset == 0 && reg.mr(0).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x1;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 1 && reg.mr(1).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x8;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 2 && reg.mr(2).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x40;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
                }
                if offset == 3 && reg.mr(3).read().bits() == 0 {
                    if self._periodic {
                        let mut data = reg.mcr().read().bits();
                        data |= 0x200;
                        reg.mcr().write(|w| unsafe { w.bits(data) });
                    }
                    return Poll::Ready(());
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

/////////// CTimer State management ///////////////////////
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

        if timer_idx == 0 {
            let reg = unsafe { Ctimer0::steal() };
            let offset = id % CHANNEL_PER_MODULE;
            let data = reg.mcr().read().bits();
            if offset == 0 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x1) });
            } else if offset == 1 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x8) });
            } else if offset == 2 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x40) });
            } else if offset == 3 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x200) });
            }
        }
        if timer_idx == 1 {
            let reg = unsafe { Ctimer1::steal() };
            let offset = id % CHANNEL_PER_MODULE;
            let data = reg.mcr().read().bits();
            if offset == 0 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x1) });
            } else if offset == 1 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x8) });
            } else if offset == 2 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x40) });
            } else if offset == 3 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x200) });
            }
        }
        if timer_idx == 2 {
            let reg = unsafe { Ctimer2::steal() };
            let offset = id % CHANNEL_PER_MODULE;
            let data = reg.mcr().read().bits();
            if offset == 0 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x1) });
            } else if offset == 1 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x8) });
            } else if offset == 2 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x40) });
            } else if offset == 3 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x200) });
            }
        }
        if timer_idx == 3 {
            let reg = unsafe { Ctimer3::steal() };
            let offset = id % CHANNEL_PER_MODULE;
            let data = reg.mcr().read().bits();
            if offset == 0 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x1) });
            } else if offset == 1 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x8) });
            } else if offset == 2 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x40) });
            } else if offset == 3 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x200) });
            }
        }
        if timer_idx == 4 {
            let reg = unsafe { Ctimer4::steal() };
            let offset = id % CHANNEL_PER_MODULE;
            let data = reg.mcr().read().bits();
            if offset == 0 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x1) });
            } else if offset == 1 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x8) });
            } else if offset == 2 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x40) });
            } else if offset == 3 {
                reg.mcr().write(|w| unsafe { w.bits(data | 0x200) });
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

        if timer_idx == 0 {
            let reg = unsafe { Ctimer0::steal() };
            let offset = (id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;
            let data = reg.ccr().read().bits();
            if offset == 0 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x4) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x1) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x2) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x3) });
                }
            } else if offset == 1 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x20) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x8) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x10) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x18) });
                }
            } else if offset == 2 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x100) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x40) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x80) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0xC0) });
                }
            } else if offset == 3 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x800) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x200) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x400) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x600) });
                }
            }
        }
        if timer_idx == 1 {
            let reg = unsafe { Ctimer1::steal() };
            let offset = (id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;
            let data = reg.ccr().read().bits();
            if offset == 0 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x4) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x1) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x2) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x3) });
                }
            } else if offset == 1 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x20) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x8) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x10) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x18) });
                }
            } else if offset == 2 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x100) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x40) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x80) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0xC0) });
                }
            } else if offset == 3 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x800) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x200) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x400) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x600) });
                }
            }
        }
        if timer_idx == 2 {
            let reg = unsafe { Ctimer2::steal() };
            let offset = (id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;
            let data = reg.ccr().read().bits();
            if offset == 0 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x4) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x1) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x2) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x3) });
                }
            } else if offset == 1 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x20) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x8) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x10) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x18) });
                }
            } else if offset == 2 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x100) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x40) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x80) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0xC0) });
                }
            } else if offset == 3 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x800) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x200) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x400) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x600) });
                }
            }
        }
        if timer_idx == 3 {
            let reg = unsafe { Ctimer3::steal() };
            let offset = (id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;
            let data = reg.ccr().read().bits();
            if offset == 0 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x4) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x1) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x2) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x3) });
                }
            } else if offset == 1 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x20) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x8) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x10) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x18) });
                }
            } else if offset == 2 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x100) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x40) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x80) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0xC0) });
                }
            } else if offset == 3 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x800) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x200) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x400) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x600) });
                }
            }
        }
        if timer_idx == 4 {
            let reg = unsafe { Ctimer4::steal() };
            let offset = (id - COUNT_CHANNEL) % CHANNEL_PER_MODULE;
            let data = reg.ccr().read().bits();
            if offset == 0 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x4) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x1) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x2) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x3) });
                }
            } else if offset == 1 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x20) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x8) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x10) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x18) });
                }
            } else if offset == 2 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x100) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x40) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x80) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0xC0) });
                }
            } else if offset == 3 {
                reg.ccr().write(|w| unsafe { w.bits(data | 0x800) });
                if edge == CaptureChEdge::Rising {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x200) });
                } else if edge == CaptureChEdge::Falling {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x400) });
                } else {
                    reg.ccr().write(|w| unsafe { w.bits(data | 0x600) });
                }
            }
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
    if inst == 0 {
        let reg = unsafe { Ctimer0::steal() };

        if reg.ir().read().mr0int().bit_is_set() {
            let mut data = reg.mcr().read().bits();
            data &= !0x1;
            reg.mcr().write(|w| unsafe { w.bits(data) });
            reg.ir().write(|w| w.mr0int().set_bit());
            reg.mr(0).write(|w| unsafe { w.match_().bits(0) });
            WAKERS[0].wake();
        }
        if reg.ir().read().mr1int().bit_is_set() {
            let mut data = reg.mcr().read().bits();
            data &= !0x8;
            reg.mcr().write(|w| unsafe { w.bits(data) });
            reg.ir().write(|w| w.mr1int().set_bit());
            reg.mr(1).write(|w| unsafe { w.match_().bits(0) });
            WAKERS[1].wake();
        }
        if reg.ir().read().mr2int().bit_is_set() {
            let mut data = reg.mcr().read().bits();
            data &= !0x40;
            reg.mcr().write(|w| unsafe { w.bits(data) });
            reg.ir().write(|w| w.mr2int().set_bit());
            WAKERS[2].wake();
        }
        if reg.ir().read().mr3int().bit_is_set() {
            let mut data = reg.mcr().read().bits();
            data &= !0x200;
            reg.mcr().write(|w| unsafe { w.bits(data) });
            reg.ir().write(|w| w.mr3int().set_bit());
            WAKERS[3].wake();
        }
        if reg.ir().read().cr0int().bit_is_set() {
            let mut data = reg.ccr().read().bits();
            data &= !0x4;
            reg.ccr().write(|w| unsafe { w.bits(data) });
            reg.ir().write(|w| w.cr0int().set_bit());
            WAKERS[20].wake();
        }
        if reg.ir().read().cr1int().bit_is_set() {
            reg.ir().write(|w| w.cr1int().set_bit());
            WAKERS[21].wake();
        }
        if reg.ir().read().cr2int().bit_is_set() {
            reg.ir().write(|w| w.cr2int().set_bit());
            WAKERS[22].wake();
        }
        if reg.ir().read().cr3int().bit_is_set() {
            reg.ir().write(|w| w.cr3int().set_bit());
            WAKERS[23].wake();
        }
    }
}

#[cfg(feature = "rt")]
#[interrupt]
#[allow(non_snake_case)]
fn CTIMER0() {
    irq_handler(0)
}

#[cfg(feature = "rt")]
#[interrupt]
#[allow(non_snake_case)]
fn CTIMER1() {
    irq_handler(1)
}

#[cfg(feature = "rt")]
#[interrupt]
#[allow(non_snake_case)]
fn CTIMER2() {
    irq_handler(2)
}

#[cfg(feature = "rt")]
#[interrupt]
#[allow(non_snake_case)]
fn CTIMER3() {
    irq_handler(3)
}

#[cfg(feature = "rt")]
#[interrupt]
#[allow(non_snake_case)]
fn CTIMER4() {
    irq_handler(4)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let ctimer = CountingTimer::new(0);
        match timer.StartOneShotTimer(100, || {
            println!("Timer Expired. Test successful !!");
        }) {
            Err(e) => panic!("Error starting timer: {:?}", e),
        }
    }
}
