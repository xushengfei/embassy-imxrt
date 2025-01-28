#![no_main]
#![no_std]

extern crate embassy_imxrt_examples;

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio;
use embassy_time::Timer;
use {defmt_rtt as _, embassy_imxrt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let embassy_p = embassy_imxrt::init(Default::default());

    info!("Initializing GPIO");

    let mut led = gpio::Output::new(
        embassy_p.PIO0_26,
        gpio::Level::Low,
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    loop {
        info!("Toggling GPIO0_26 (Blue LED)");
        led.toggle();

        #[cfg(feature = "test-parser")]
        test_parser_macros::pass_test();

        Timer::after_millis(5000).await;
    }
}
