#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use panic_probe as _;

use embassy_imxrt::wwdt::WindowedWatchdog;

// Prevent CPU from going to sleep while waiting for watchdog interrupt
#[embassy_executor::task]
async fn caffeine() {
    loop {
        embassy_futures::yield_now().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(caffeine()).unwrap();

    let p = embassy_imxrt::init(Default::default());
    let mut wwdt = WindowedWatchdog::new(p.WDT0, 1_000_000);
    wwdt.clear_timeout_flag();
    wwdt.enable_reset()
        .lock()
        .protect_timeout()
        .set_warning_threshold(4_096);

    let mut wwdt = wwdt.unleash();
    info!("WWDT enabled!");

    let mut feed_count = 5;
    loop {
        info!("Waiting for watchdog warning...");
        wwdt.wait_for_warning().await;
        info!("Warning! Timeout in: {} us", wwdt.timeout());

        // Feed 5 times, afterwards watchdog will reset CPU
        if feed_count > 0 {
            wwdt.feed();
            info!("Watchdog fed... for now");
            feed_count -= 1;
        }
    }
}
