#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::time_driver::*;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    let r = RtcDatetime::new(p.RTC);

    #[cfg(feature = "test-parser")]
    test_parser_macros::pass_test();

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
    assert!(ret.is_ok());

    info!("Wait for 20 seconds");
    //wait for 20 seconds
    Timer::after_millis(20000).await;

    // get the datetime set and compare
    let (time, result) = r.get_datetime();
    assert!(result.is_ok());
    info!("RTC get time: {:?}", time);
}
