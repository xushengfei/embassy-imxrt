#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_imxrt::init(Default::default());

    info!("Hello world");

    #[cfg(feature = "test-parser")]
    test_parser_macros::pass_test();

    loop {
        Timer::after_millis(1000).await;
    }
}
