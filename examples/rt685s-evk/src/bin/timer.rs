#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::bind_interrupts;
use embassy_imxrt::timer;
use embassy_imxrt::timer::{Countdown, Timer};

// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    //info!("Timer example - Starting");

    let mut timer_manager: timer::CTimerManager<timer::Uninitialized> = timer::CTimerManager::new();

    let mut timer_manager: timer::CTimerManager<timer::Initialized> = timer_manager.init_timer_modules();

    //info!("Timer example - clk enable Value for CTIMER0 = {:02X}", result as u32);
    // info!(
    //     "Timer example - clock enable Value for CTIMER0 = {:02X}",
    //     result.1 as u32
    // );

    let mut tmr1 = timer_manager.request_counting_timer(|| {
        info!("Timer1 example - Timer Callback");
    });

    let mut tmr2 = timer_manager.request_counting_timer(|| {
        info!("Timer2 example - Timer Callback");
    });

    tmr1.start_timer(5000000);

    tmr2.start_timer(10000000);

    let result = timer_manager.read_timer_registers();

    // info!("Clock source = {:02X}", result.0);
    // info!("ctimer0 tr = {:02X}", result.1);
    // info!("ctimer0 mr1 = {:02X}", result.2);
    // info!("ctimer0 mr0 = {:02X}", result.3);
    info!("ctimer0 mcr = {:02X}", result.4);
    // info!("ctimer0 pcr = {:02X}", result.4);
    // info!("ctimer0 pr = {:02X}", result.5);
    tmr1.wait().await;
    tmr2.wait().await;

    loop {
        embassy_imxrt_examples::delay(50000);
        info!("CTimer1 MR - 0 - {:02X}", timer_manager.read_irq_reg().0);
        info!("CTimer1 MR - 1 - {:02X}", timer_manager.read_irq_reg().1);
        info!("CTimer0 MCR = {:02X}", timer_manager.read_timer_registers().4);
    }
}
