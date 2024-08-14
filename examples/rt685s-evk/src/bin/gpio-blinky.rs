#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio::{Level, Output, OutputDrive};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("started running");
    let p = embassy_imxrt::init(Default::default());

    let mut led = Output::new(p.PIO0_26, Level::Low, OutputDrive::Normal);

    loop {
        info!("Toggling GPIO");
        led.toggle();
        embassy_imxrt_examples::delay(50_000);
    }
}
