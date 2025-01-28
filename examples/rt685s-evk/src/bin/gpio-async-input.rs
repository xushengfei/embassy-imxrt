#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::gpio;
use embassy_time::{Duration, Ticker};

#[embassy_executor::task]
async fn monitor_task(mut monitor: gpio::Input<'static>) {
    loop {
        monitor.wait_for_falling_edge().await;
        debug!("3 Falling edge detected");

        monitor.wait_for_low().await;
        debug!("4 Level low detected");

        monitor.wait_for_high().await;
        debug!("6 Level high detected");

        monitor.wait_for_rising_edge().await;
        debug!("9 Rising edge detected");

        monitor.wait_for_any_edge().await;
        debug!("11 An any (rising) edge detected");

        monitor.wait_for_any_edge().await;
        debug!("13 An any (falling) edge detected");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    debug!("Initializing GPIO");

    let mut output = gpio::Output::new(
        p.PIO1_2,
        gpio::Level::Low,
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    let monitor = gpio::Input::new(p.PIO1_0, gpio::Pull::None, gpio::Inverter::Disabled);

    let mut ticker = Ticker::every(Duration::from_millis(100));

    #[cfg(feature = "test-parser")]
    test_parser_macros::pass_test();

    spawner.spawn(monitor_task(monitor)).unwrap();

    loop {
        debug!("1 Output is low");
        ticker.next().await;

        output.set_high();
        output.set_low();
        debug!("2 Output go high -> low");
        ticker.next().await;

        output.set_high();
        debug!("5 Output go high");
        ticker.next().await;

        debug!("7 Output is high");
        ticker.next().await;

        output.set_low();
        output.set_high();
        output.set_low();
        debug!("8 Output go low -> high -> low");
        ticker.next().await;

        output.set_high();
        debug!("10 Output go high");
        ticker.next().await;

        output.set_low();
        debug!("12 Output go low");
        ticker.next().await;
    }
}
