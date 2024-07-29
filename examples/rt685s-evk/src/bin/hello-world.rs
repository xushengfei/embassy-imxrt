#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_imxrt::init(Default::default());

    info!("Hello world");

    loop {
        embassy_imxrt_examples::delay(50_000_000);
    }
}
