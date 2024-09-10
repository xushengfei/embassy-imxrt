#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio::{self, *};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("started running");
    let p = embassy_imxrt::init(Default::default());

    info!("initializing GPIO");
    Port::init(Port::Port0); // to enable GPIO port 0

    // default pin configuration
    let mut pin_config: gpio::Config = Config::new();
    // setting the initial output level as Normal
    pin_config.drive_strength = DriveStrength::Normal;

    let mut led = gpio::Output::new(p.PIO0_26, pin_config);

    loop {
        info!("Toggling GPIO");
        led.toggle();
        embassy_imxrt_examples::delay(50_000);
    }
}
