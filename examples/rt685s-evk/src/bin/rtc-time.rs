#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::time_driver::Datetime;
use embassy_imxrt::time_driver::RtcDatetime;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_imxrt::init(Default::default());
    let r = RtcDatetime {
        datetime: Datetime {
            year: 2021,
            month: 9,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0,
        },
    };
    r.set_datetime();
    //info!("Setting RTC time to {:?}", datetime.day);
    embassy_imxrt_examples::delay(50000);
    let time = r.get_datetime();
    info!("RTC time is {:?}", time);
    embassy_imxrt_examples::delay(50000);
}
