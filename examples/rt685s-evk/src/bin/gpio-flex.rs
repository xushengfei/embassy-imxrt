#![no_std]
#![no_main]

use defmt::assert;
use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing GPIO");
    unsafe { gpio::init() };

    let mut flex = gpio::Flex::new(p.PIO1_0);

    // set pin output bit to high before setting direction
    flex.set_high();

    // set direction as output
    flex.set_as_output(
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    // check pin level is high
    assert!(flex.is_high());

    // toggle pin
    flex.toggle();

    // check pin level is low
    assert!(flex.is_low());

    // set pin direction to output with reverse polarity
    flex.set_as_input(gpio::Pull::None, gpio::Polarity::ActiveLow);

    // check pin level is high
    assert!(flex.is_high());

    // set pin output bit to high
    flex.set_high();

    // check pin level is still high
    assert!(flex.is_high());

    // set pin direction to output again
    flex.set_as_output(
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    // check pin level is now low
    assert!(flex.is_low());

    loop {
        embassy_imxrt_examples::delay(50_000);
    }
}
