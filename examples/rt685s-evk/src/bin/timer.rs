#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::clocks::ClockConfig;
use embassy_imxrt::timer::{CaptureChEdge, CaptureTimer, CountingTimer, TriggerInput};
use embassy_imxrt::{bind_interrupts, peripherals, timer};
use embassy_time::Timer as Tmr;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CTIMER0 => timer::CtimerInterruptHandler<peripherals::CTIMER0_COUNT_CHANNEL0>;
    CTIMER1 => timer::CtimerInterruptHandler<peripherals::CTIMER1_COUNT_CHANNEL0>;
    CTIMER2 => timer::CtimerInterruptHandler<peripherals::CTIMER2_COUNT_CHANNEL0>;
    CTIMER3 => timer::CtimerInterruptHandler<peripherals::CTIMER3_COUNT_CHANNEL0>;
    CTIMER4 => timer::CtimerInterruptHandler<peripherals::CTIMER4_COUNT_CHANNEL0>;
});

// Monitor task is created to demonstrate difference between Async and Blocking timer behavior
#[embassy_executor::task]
async fn monitor_task() {
    loop {
        info!("Secondary task running");
        Tmr::after_millis(1000).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    _spawner.spawn(monitor_task()).unwrap();

    let sfro = ClockConfig::crystal().sfro;
    let mut tmr1 = CountingTimer::new_blocking(p.CTIMER0_COUNT_CHANNEL0, sfro);

    let sfro = ClockConfig::crystal().sfro;
    let mut tmr2 = CountingTimer::new_async(p.CTIMER1_COUNT_CHANNEL0, sfro);

    tmr1.wait(3000000); // 3 seoconds wait
    info!("First Counting timer expired");

    tmr2.wait(5000000).await; //  5 seconds wait
    info!("Second Counting timer expired");

    // Creating a separate block to test Timer Drop logic
    {
        let sfro = ClockConfig::crystal().sfro;
        let mut cap_async_tmr = CaptureTimer::new_async(p.CTIMER0_CAPTURE_CHANNEL0, sfro);

        // pass the input mux number user is interested in
        // Input mux details can be found in NXP user manual section 8.6.8 and Pin Function Table in section 7.5.3
        cap_async_tmr.start(TriggerInput::TrigIn9, p.PIO1_7, CaptureChEdge::Falling);

        cap_async_tmr.wait().await;
        info!(
            "Capture timer expired in = {} ms",
            cap_async_tmr.get_event_capture_time_ms()
        );
    }

    loop {
        tmr2.wait(5000000).await;
        info!("Primary task running");
    }
}
