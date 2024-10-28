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
    let p = embassy_imxrt::init(Default::default());

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

    let cap_tmr = timer_manager.request_capture_timer(
        |count_reg| {
            info!("Capture Timer2 example - Capture Timer Callback");
            info!("count reg = {:02x}", count_reg);
        },
        CaptureChEdge::Rising,
        false,
    );

    let pac = embassy_imxrt::pac::Peripherals::take().unwrap();

    pac.iopctl.pio1_0().write(|w| w.fsel().function_4());

    let monitor = gpio::Input::new(p.PIO1_0, gpio::Pull::None, gpio::Polarity::ActiveLow);

    tmr1.start_count(5000000);
    tmr2.start_count(10000000);
    cap_tmr.start_capture(8); // pass the input mux number user is interested in

    tmr1.wait().await;
    tmr2.wait().await;
    cap_tmr.wait().await;

    loop {
        info!("Pin level is {}", monitor.get_level());
        Tmr::after_millis(500).await;
    }
}
