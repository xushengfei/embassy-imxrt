#![no_std]
#![no_main]

// #[embedded_test::tests]
#[cfg(test)]
#[embedded_test::tests]
mod tests {
    use defmt::info;
    use defmt_rtt as _;
    use embassy_imxrt;
    use embassy_imxrt::gpio;
    use embassy_time::Timer;
    // Optional: A init function which is called before every test
    // asyncness of init fn is optional
    #[init]
    fn init() -> gpio::Output<'static> {
        let p = embassy_imxrt::init(Default::default());

        info!("Initializing GPIO");
        unsafe { gpio::init() };

        let led = gpio::Output::new(
            p.PIO0_26,
            gpio::Level::Low,
            gpio::DriveMode::PushPull,
            gpio::DriveStrength::Normal,
            gpio::SlewRate::Standard,
        );
        led
    }

    // // A test which takes the state returned by the init function (optional)
    #[test]
    async fn test_blink(mut led: gpio::Output<'static>) {
        for _ in 1..5 {
            Timer::after_millis(500).await;
            led.toggle();
        }
        info!("It works");
        assert!(true);
    }

    // // A test which takes the state returned by the init function (optional)
    #[test]
    #[should_panic]
    async fn test_should_panic(mut led: gpio::Output<'static>) {
        for _ in 1..5 {
            Timer::after_millis(500).await;
            led.toggle();
        }
        info!("It works");
        assert!(false);
    }
}
