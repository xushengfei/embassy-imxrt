#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::adc::Adc;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    let _adc = Adc::new(p.ADC0);

    loop {
        info!("I am the ADC example");
        embassy_imxrt_examples::delay(50_000_000);
    }
}
