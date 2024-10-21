#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use cortex_m::peripheral::NVIC;
use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::pac::{interrupt, Interrupt};
use embassy_imxrt::wwdt::WindowedWatchdog;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    let mut wwdt = WindowedWatchdog::new(p.WDT0, 1_000_000);
    wwdt.clear_timeout_flag();
    wwdt.enable_reset().lock().set_warning_threshold(4_096);

    unsafe { NVIC::unmask(Interrupt::WDT0) };

    let mut wwdt = wwdt.unleash();
    info!("Watchdog enabled!");

    // Feed 5 times, afterwards watchdog will reset CPU
    let mut feed_count = 5;
    loop {
        if feed_count > 0 {
            wwdt.feed();
            feed_count -= 1;
            embassy_imxrt_examples::delay(25_000);
            info!("Reset in {} μs if feed does not occur", wwdt.timeout());
        }
    }
}

#[interrupt]
fn WDT0() {
    /* This may not appear in logger since there may not be enough time
     * for transfer to complete before reset.
     */
    warn!("System reset imminent!");
}
