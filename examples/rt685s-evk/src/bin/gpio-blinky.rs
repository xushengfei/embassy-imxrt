#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio::{self, *};
use embassy_imxrt::iopctl::DriveStrength;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("started running");
    let p = embassy_imxrt::init(Default::default());

    Port::init(Port::Port0); // to enable GPIO port 0

    // default pin configuration
    let mut pin_config: gpio::Config = Default::default();
    // setting the initial output level as Normal
    pin_config.drive_strength = DriveStrength::Normal;

    let mut led = gpio::Output::new(p.PIO0_26, pin_config);

    // let mut cnt = 0;
    loop {
        info!("Toggling GPIO");
        led.toggle();
        embassy_imxrt_examples::delay(50_000);
        // cnt += 1;
        // if cnt == 20 {
        //     info!("disconnecting GPIO pin");
        //     led.set_as_disconnected();
        // }
    }
}
