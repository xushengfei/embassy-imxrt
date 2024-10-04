#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::time_driver::*;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_imxrt::init(Default::default());
    let r = RtcDatetime::new();

    let mut datetime = Datetime {
        year: 2024,
        month: 10,
        day: 4,
        hour: 16,
        minute: 0,
        second: 0,
    };
    let ret = r.set_datetime(&datetime);
    // check if the set is valid
    assert!(ret == DatetimeResult::ValidDatetime);

    //wait for 20 seconds
    Timer::after_millis(20000).await;

    // get the datetime set and compare
    let time = r.get_datetime();
    info!("RTC time is {:?}", time);

    embassy_imxrt_examples::delay(50000);
}
