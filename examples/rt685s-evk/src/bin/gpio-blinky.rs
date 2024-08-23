#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::gpio::{Port, Level, Output, OutputDrive};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("started running");
    let p = embassy_imxrt::init(Default::default());

    Port::init(Port::Port0); // to enable GPIO port 0
    let mut led = Output::new(p.PIO0_26, Level::Low, OutputDrive::Normal);

    let mut cnt =0;
    loop {
        info!("Toggling GPIO");
        led.toggle();
        embassy_imxrt_examples::delay(50_000);
        cnt+=1;
        if cnt==20{
            break;
        }
    }
    led.set_as_disconnected();

}
