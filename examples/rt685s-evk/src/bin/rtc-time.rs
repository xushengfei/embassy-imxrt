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

    let datetime = Datetime {
        year: 2024,
        month: 10,
        day: 4,
        hour: 16,
        minute: 0,
        second: 0,
    };
    let ret = r.set_datetime(&datetime);
    info!("RTC set time: {:?}", datetime);
    // check if the set is valid
    assert!(ret == DatetimeResult::ValidDatetime);

    info!("Wait fo 20 seconds");
    //wait for 20 seconds
    Timer::after_millis(20000).await;

    // get the datetime set and compare
    let (time, result) = r.get_datetime();
    assert!(result == DatetimeResult::ValidDatetime);
    info!("RTC get time: {:?}", time);

    embassy_imxrt_examples::delay(50000);
}
