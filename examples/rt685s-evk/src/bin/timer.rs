#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::bind_interrupts;
use embassy_imxrt::timer;
use embassy_imxrt::timer::{Countdown, Timer};
use embassy_time::Timer as Tmr;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    let mut timer_manager: timer::CTimerManager<timer::Uninitialized> = timer::CTimerManager::new();

    let mut timer_manager: timer::CTimerManager<timer::Initialized> = timer_manager.init_timer_modules();

    let mut tmr1 = timer_manager.request_counting_timer(
        || {
            info!("Timer1 example - Timer Callback");
        },
        true,
    );

    let mut tmr2 = timer_manager.request_counting_timer(
        || {
            info!("Timer2 example - Timer Callback");
        },
        true,
    );

    tmr1.start_timer(5000000);
    tmr2.start_timer(10000000);

    tmr1.wait().await;
    tmr2.wait().await;

    loop {
        tmr1.wait().await;
        tmr2.wait().await;
    }
}
