use core::cell::Cell;
use core::sync::atomic::{compiler_fence, AtomicU32, AtomicU8, Ordering};
use core::{mem, ptr, u32, u64};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy_time_driver::{AlarmHandle, Driver};

use crate::interrupt::InterruptExt;
use crate::{interrupt, pac};

fn rtc() -> &'static pac::rtc::RegisterBlock {
    unsafe { &*pac::Rtc::ptr() }
}

/// Calculate the timestamp from the period count and the tick count.
///
/// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
/// the expected range for the `period` parity, we're done. If it doesn't, this means that
/// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
/// corresponds to the next period.
///
/// the 1kHz RTC counter is 16 bits and RTC doesn't have separate compare channels,
/// so using a 32 bit GPREG0-2 as counter, compare, and int_en
/// `period` is a 32bit integer, gpreg 'counter' is 31 bits plus the parity bit for overflow detection
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 31) + ((counter ^ ((period & 1) << 31)) as u64)
}

struct AlarmState {
    timestamp: Cell<u64>,
    // This is really a Option<(fn(*mut ()), *mut ())>
    // but fn pointers aren't allowed in const yet
    callback: Cell<*const ()>,
    ctx: Cell<*mut ()>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(ptr::null()),
            ctx: Cell::new(ptr::null_mut()),
        }
    }
}

const ALARM_COUNT: usize = 1;

struct TimerDriver {
    /// Number of 2^31 periods elapsed since boot.
    period: AtomicU32,
    alarm_count: AtomicU8,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();
embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver {
    period: AtomicU32::new(0),
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [ALARM_STATE_NEW; ALARM_COUNT]),
});

impl TimerDriver {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = rtc();
        // enable RTC int (1kHz since subsecond doesn't generate an int)
        r.ctrl().modify(|_r, w| w.rtc1khz_en().set_bit());
        // TODO: low power support. line above is leaving out write to .wakedpd_en().set_bit())
        // which enables wake from deep power down

        // safety: Writing to the gregs is always considered unsafe, gpreg1 is used
        // as a compare register for triggering an alarm so to avoid unnecessary triggers
        // after initialization, this is set to 0x:FFFF_FFFF
        r.gpreg(1).write(|w| unsafe { w.gpdata().bits(u32::MAX) });
        interrupt::RTC.set_priority(irq_prio);
        unsafe { interrupt::RTC.enable() };
    }

    fn on_interrupt(&self) {
        let r = rtc();
        // This interrupt fires every 10 ticks of the 1kHz RTC high res clk and adds
        // 10 to the 31 bit counter gpreg0. The 32nd bit is used for parity detection
        // This is done to avoid needing to calculate # of ticks spent on interrupt
        // handlers to recalibrate the clock between interrupts
        //
        // TODO: this is admittedly not great for power that we're generating this
        // many interrupts, will probably get updated in future iterations.
        if r.ctrl().read().wake1khz().bit_is_set() == true {
            r.ctrl().modify(|_r, w| w.wake1khz().set_bit());
            // safety: writing a value to the 1kHz RTC wake counter is always considered unsafe.
            // The following reloads 10 into the count-down timer after it triggers an int.
            // The countdown begins anew after the write so time can continue to be measured.
            r.wake().write(|w| unsafe { w.bits(0xA) });
            if (r.gpreg(0).read().bits() + 0xA) > 0x8000_0000 {
                // if we're going to "overflow", increase the period
                self.next_period();
                let rollover_diff = 0x8000_0000 - (r.gpreg(0).read().bits() + 0xA);
                // safety: writing to gpregs is always considered unsafe. In order to
                // not "lose" time when incrementing the period, gpreg0, the extended
                // counter, is restarted at the # of ticks it would overflow by
                r.gpreg(0).write(|w| unsafe { w.bits(rollover_diff) });
            } else {
                r.gpreg(0).modify(|r, w| unsafe { w.bits(r.bits() + 0xA) });
            }
        }

        critical_section::with(|cs| {
            // gpreg2 as an "int_en" set by next_period(). This is
            // 1 when the timestamp for the alarm deadline expires
            // before the counter register overflows again.
            if r.gpreg(2).read().gpdata().bits() == 1 {
                // gpreg0 is our extended counter register, check if
                // our counter is larger than the compare value
                if r.gpreg(0).read().bits() > r.gpreg(1).read().bits() {
                    self.trigger_alarm(0, cs);
                }
            }
        })
    }

    fn next_period(&self) {
        critical_section::with(|cs| {
            let r = rtc();
            let period = self
                .period
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |p| Some(p + 1))
                .unwrap_or_else(|p| {
                    trace!("Unable to increment period. Time is now inaccurate");
                    // TODO: additional error handling beyond logging

                    p
                });
            let t = (period as u64) << 31;

            let alarm = &self.alarms.borrow(cs)[0];
            let at = alarm.timestamp.get();
            if at < t + 0xc000_0000 {
                // safety: writing to gpregs is always unsafe, gpreg2 is an alarm
                // enable. If the alarm must trigger within the next period, then
                // just enable it. `set_alarm` has already set the correct CC val.
                r.gpreg(2).write(|w| unsafe { w.gpdata().bits(1) });
            }
        })
    }

    fn get_alarm<'a>(&'a self, cs: CriticalSection<'a>, alarm: AlarmHandle) -> &'a AlarmState {
        // safety: we're allowed to assume the AlarmState is created by us, and
        // we never create one that's out of bounds.
        unsafe { self.alarms.borrow(cs).get_unchecked(alarm.id() as usize) }
    }

    fn trigger_alarm(&self, n: usize, cs: CriticalSection) {
        let r = rtc();
        // safety: writing to gpregs is always unsafe. Because
        // gpreg 2 is "int_en" and gpreg1 is the compare register,
        // after we trigger an alarm, the enable must be cleared and
        // our compare must go back to the initialization value
        r.gpreg(2).write(|w| unsafe { w.bits(0) });
        r.gpreg(1).write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        let alarm = &self.alarms.borrow(cs)[n];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.

        // safety:
        // - we can ignore the possiblity of `f` being unset (null) because of the safety contract of `allocate_alarm`.
        // - other than that we only store valid function pointers into alarm.callback
        let f: fn(*mut ()) = unsafe { mem::transmute(alarm.callback.get()) };
        f(alarm.ctx.get());
    }
}

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        // `period` MUST be read before `counter`, see comment at the top for details.
        let period = self.period.load(Ordering::Acquire);
        compiler_fence(Ordering::Acquire);
        let counter = rtc().gpreg(0).read().bits();
        calc_now(period, counter)
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        critical_section::with(|_| {
            let id = self.alarm_count.load(Ordering::Relaxed);
            if id < ALARM_COUNT as u8 {
                self.alarm_count.store(id + 1, Ordering::Relaxed);
                Some(AlarmHandle::new(id))
            } else {
                None
            }
        })
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);

            alarm.callback.set(callback as *const ());
            alarm.ctx.set(ctx);
        })
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);

            let r = rtc();

            let t = self.now();
            if timestamp <= t {
                // safety: Writing to the gpregs is always unsafe, gpreg2 is
                // always just used as the alarm enable for the timer driver.
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                r.gpreg(2).write(|w| unsafe { w.gpdata().bits(0) });

                alarm.timestamp.set(u64::MAX);

                return false;
            }

            // If it hasn't triggered yet, setup it by writing to the compare field
            // An alarm can be delayed, but this is allowed by the Alarm trait contract.
            // What's not allowed is triggering alarms *before* their scheduled time,
            let safe_timestamp = timestamp.max(t + 10); //t+3 was done for nrf chip, choosing 10

            // safety: writing to the gregs is always unsafe. When a new alarm is set,
            // the compare register, gpreg1, is set to the last 31 bits of the timestamp
            // as the 32nd and final bit is used for the parity check in `next_period`
            // `period` will be used for the upper bits in a timestamp comparison.
            r.gpreg(1)
                .modify(|_r, w| unsafe { w.bits(safe_timestamp as u32 & 0x7FFF_FFFF) });

            // The following checks that the difference in timestamp is less than the overflow period
            let diff = timestamp - t;
            if diff < 0xc000_0000 {
                // this is 0b11 << (30). NRF chip used 23 bit periods and checked against 0b11<<22

                // safety: writing to the gpregs is always unsafe. If the alarm
                // must trigger within the next period, set the "int enable"
                r.gpreg(2).write(|w| unsafe { w.gpdata().bits(1) });
            } else {
                // safety: writing to the gpregs is always unsafe. If alarm must trigger
                // some time after the current period, too far in the future, don't setup
                // the alarm enable, gpreg2, yet. It will be setup later by `next_period`.
                r.gpreg(2).write(|w| unsafe { w.gpdata().bits(0) });
            }

            true
        })
    }
}

#[cfg(feature = "rt")]
#[interrupt]
fn RTC() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}
