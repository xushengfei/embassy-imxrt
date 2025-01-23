#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing GPIO");

    let monitor = gpio::Input::new(p.PIO1_0, gpio::Pull::None, gpio::Inverter::Disabled);

    #[cfg(feature = "test-parser")]
    test_parser_macros::pass_test();

    loop {
        info!("Pin level is {}", monitor.get_level());
        Timer::after_millis(1000).await;
    }
}
