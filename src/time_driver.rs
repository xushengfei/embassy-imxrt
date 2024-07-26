use core::cell::Cell;
use core::sync::atomic::{compiler_fence, AtomicU32, AtomicU8, Ordering};
use core::{mem, ptr};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
//TODO
use embassy_time_driver::{AlarmHandle, Driver}; //cargo build can't find this

use crate::interrupt::InterruptExt;
use crate::{interrupt, pac};

fn rtc() -> &'static pac::rtc::RegisterBlock {
    unsafe { &*pac::Rtc::ptr() }
}
fn timer0() -> &'static pac::ctimer0::RegisterBlock {
    unsafe { &*pac::Ctimer0::ptr() }
}
fn timer1() -> &'static pac::ctimer1::RegisterBlock {
    unsafe { &*pac::Ctimer1::ptr() }
}
fn timer2() -> &'static pac::ctimer2::RegisterBlock {
    unsafe { &*pac::Ctimer2::ptr() }
}
fn timer3() -> &'static pac::ctimer3::RegisterBlock {
    unsafe { &*pac::Ctimer3::ptr() }
}
fn timer4() -> &'static pac::ctimer4::RegisterBlock {
    unsafe { &*pac::Ctimer4::ptr() }
}
/// Calculate the timestamp from the period count and the tick count.
///
/// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
/// the expected range for the `period` parity, we're done. If it doesn't, this means that
/// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
/// corresponds to the next period.
///
/// `period` is a 32bit integer,
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

const ALARM_COUNT: usize = 4;

struct TimerDriver {
    /// Number of 2^32 periods elapsed since boot.
    period: AtomicU32,
    alarm_count: AtomicU8,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();
// error since embassy_time_driver not found
embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver {
    period: AtomicU32::new(0),
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [ALARM_STATE_NEW; ALARM_COUNT]),
});

impl TimerDriver {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = rtc();
        let t0 = timer0();

        //enable timer reset on int and interrupts
        // should we clear on int if we're using the same timer but different alarms?

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
        });
        //enable rtc clk
        r.ctrl().modify(|_r, w| w.rtc_en().set_bit());
        //enable subsecond ticking so it actually counts at 32kHz instead of 1Hz
        r.ctrl().modify(|_r, w| w.rtc_subsec_ena().set_bit()); //??
                                                               //enable RTC int (1Hz or 1kHz since subsecond doesn't generate an int?)
        r.ctrl()
            .modify(|_r, w| w.rtc1khz_en().set_bit().wakedpd_en().set_bit());

        // reset timer counters and then start them
        t0.tcr().modify(|_r, w| w.crst().set_bit());
        /*t1.tcr().modify(|_r,w| w.crst().set_bit());
        t2.tcr().modify(|_r,w| w.crst().set_bit());
        t3.tcr().modify(|_r,w| w.crst().set_bit());*/

        // Wait for counters to clear
        // probably don't need to wait for each timer counter to reset?
        while r.count().read().bits() != 0 {} // will this work or will the RTC have already ticked by this point?
        while t0.tc().read().bits() != 0 {}
        /*while t1.tc().read().bits() != 0 {}
        while t2.tc().read().bits() != 0 {}
        while t3.tc().read().bits() != 0 {}*/
        // clear reset bit
        t0.tcr().modify(|_r, w| w.crst().clear_bit());
        /*t1.tcr().modify(|_r,w| w.crst().clear_bit());
        t2.tcr().modify(|_r,w| w.crst().clear_bit());
        t3.tcr().modify(|_r,w| w.crst().clear_bit());*/
        //clear the interrupts
        t0.ir().modify(|_r, w| unsafe { w.bits(0) });

        interrupt::RTC.set_priority(irq_prio);
        unsafe { interrupt::RTC.enable() };
    }

    fn on_interrupt(&self) {
        let r = rtc();
        let t0 = timer0();
        //compare rtc mask
        if r.ctrl().read().alarm1hz().bit_is_set() == true {
            r.ctrl().modify(|_r, w| w.alarm1hz().set_bit());
            //need to reset the rtc counter register?
            self.next_period();
        }
        //compare mask for all other alarms
        for n in 0..ALARM_COUNT {
            if (t0.ir().read().bits() & (1 << n)) != 0 {
                t0.ir().modify(|_r, w| unsafe { w.bits(1 << n) });
                critical_section::with(|cs| {
                    self.trigger_alarm(n, cs);
                })
            }
        }
    }

    fn next_period(&self) {
        critical_section::with(|_cs| {
            let period = self.period.load(Ordering::Relaxed) + 1;
            self.period.store(period, Ordering::Relaxed);
            //let t = (period as u64) << 31;
        })
    }

    fn get_alarm<'a>(&'a self, cs: CriticalSection<'a>, alarm: AlarmHandle) -> &'a AlarmState {
        // safety: we're allowed to assume the AlarmState is created by us, and
        // we never create one that's out of bounds.
        unsafe { self.alarms.borrow(cs).get_unchecked(alarm.id() as usize) }
    }

    fn trigger_alarm(&self, n: usize, cs: CriticalSection) {
        let t0 = timer0();
        t0.ir().modify(|_r, w| unsafe { w.bits(1 << n) });

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
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = rtc().count().read().bits();
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
            let n = alarm.id();
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);

            let t0 = timer0();

            let t = self.now();
            if timestamp <= t {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                t0.ir().modify(|_r, w| unsafe { w.bits(1 << n) });

                alarm.timestamp.set(u64::MAX);

                return false;
            }

            let safe_timestamp = timestamp.max(t + 3); //+3 was done for nrf chip
                                                       //r.cc[n].write(|w| unsafe { w.bits(safe_timestamp as u32 & 0xFFFFFF) });
            t0.tc()
                .modify(|_r, w| unsafe { w.bits(safe_timestamp as u32 & 0xFFFFFF) });

            let diff = timestamp - t;
            if diff < 0xc00000 {
                //TODO
                //set interrupt but nxp chip doesn't allow manual setting
                //t0.intenset.write(|w| unsafe { w.bits(1<<n) });
            } else {
                // If it's too far in the future, don't setup the compare channel yet.
                // It will be setup later by `next_period`.
                t0.ir().modify(|_r, w| unsafe { w.bits(1 << n) });
            }

            true
        })
    }
}

#[cfg(feature = "rt")]
#[interrupt]
fn CTIMER0() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}
