#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
// use embassy_imxrt::bind_interrupts;
use embassy_imxrt::gpio;
use embassy_imxrt::timer;
use embassy_imxrt::timer::{CaptureChEdge, Timer};
use embassy_time::Timer as Tmr;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_imxrt::init(Default::default());

    unsafe { gpio::init() };

    let timer_manager: timer::CTimerManager<timer::Uninitialized> = timer::CTimerManager::new();

    let mut timer_manager: timer::CTimerManager<timer::Initialized> = timer_manager.init_timer_modules();

    let mut tmr1 = timer_manager.request_counting_timer(
        || {
            info!("Timer1 example - Timer Callback");
        },
        false,
    );

    let mut tmr2 = timer_manager.request_counting_timer(
        || {
            info!("Timer2 example - Timer Callback");
        },
        false,
    );
    let mut tmr3 = timer_manager.request_counting_timer(
        || {
            info!("Timer3 example - Timer Callback");
        },
        false,
    );
    let mut tmr4 = timer_manager.request_counting_timer(
        || {
            info!("Timer4 example - Timer Callback");
        },
        false,
    );
    let mut tmr5 = timer_manager.request_counting_timer(
        || {
            info!("Timer5 example - Timer Callback");
        },
        false,
    );
    let mut tmr6 = timer_manager.request_counting_timer(
        || {
            info!("Timer6 example - Timer Callback");
        },
        false,
    );
    let mut tmr7 = timer_manager.request_counting_timer(
        || {
            info!("Timer7 example - Timer Callback");
        },
        false,
    );

    let cap_tmr = timer_manager.request_capture_timer(
        |count_reg| {
            info!("Capture Timer example - Capture Timer Callback");
            info!("count reg = 0x{:02X}", count_reg);
        },
        CaptureChEdge::Falling,
        false,
    );

    let pac = embassy_imxrt::pac::Peripherals::take().unwrap();

    pac.iopctl.pio1_7().modify(|_, w| w.iiena().enabled()); // Active low input polarity
    pac.iopctl.pio1_7().modify(|_, w| w.ibena().enabled()); // Input buffer enable
    pac.iopctl.pio1_7().modify(|_, w| w.fsel().function_4()); // Set Function 4

    tmr1.start_count(1000000); // 1 sec
    tmr2.start_count(2000000); // 2 sec
    tmr3.start_count(3000000); // 3 sec
    tmr4.start_count(4000000); // 4 sec
    tmr5.start_count(5000000); // 5 sec
    tmr6.start_count(6000000); // 6 sec
    tmr7.start_count(7000000); // 7 sec

    cap_tmr.start_capture(9); // pass the input mux number user is interested in

    tmr1.wait().await;
    tmr2.wait().await;
    tmr3.wait().await;
    tmr4.wait().await;
    tmr5.wait().await;
    tmr6.wait().await;
    tmr7.wait().await;

    cap_tmr.wait().await;

    loop {
        Tmr::after_millis(1000).await;
        cap_tmr.wait().await;
    }
}
