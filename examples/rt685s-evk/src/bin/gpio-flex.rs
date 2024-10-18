#![no_std]
#![no_main]

use defmt::assert;
use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio;
use embassy_imxrt::gpio::{SenseDisabled, SenseEnabled};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing GPIO");
    unsafe { gpio::init() };

    let mut flex = gpio::Flex::<SenseDisabled>::new(p.PIO1_0);

    // set pin output bit to high before setting direction
    flex.set_high();

    let flex = flex.set_as_input(gpio::Pull::None, gpio::Polarity::ActiveHigh);

    // check pin level is high
    assert!(flex.is_high());

    let mut flex = flex.set_as_output(
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    // toggle pin
    flex.toggle();

    let flex = flex.set_as_input(gpio::Pull::None, gpio::Polarity::ActiveHigh);

    // check pin level is low
    assert!(flex.is_low());

    let flex = flex.set_as_output(
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    // set pin direction to output with reverse polarity
    let flex = flex.set_as_input(gpio::Pull::None, gpio::Polarity::ActiveLow);

    // check pin level is high
    assert!(flex.is_high());

    let mut flex = flex.set_as_output(
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    // set pin output bit to high
    flex.set_high();

    let flex = flex.set_as_input(gpio::Pull::None, gpio::Polarity::ActiveLow);

    // check pin level is still high
    assert!(flex.is_high());

    // set pin direction to output again
    let mut flex = flex.set_as_output(
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    let flex = flex.set_as_input(gpio::Pull::None, gpio::Polarity::ActiveHigh);

    // check pin level is now low
    assert!(flex.is_low());

    loop {
        embassy_imxrt_examples::delay(50_000);
    }
}
