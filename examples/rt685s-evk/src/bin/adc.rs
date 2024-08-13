#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::adc::Adc;
use embassy_imxrt::adc::ChannelConfig;
use embassy_imxrt::adc::Config;
use embassy_imxrt::adc::InterruptHandler;
use embassy_imxrt::bind_interrupts;

bind_interrupts!(struct Irqs {
    ADC0 => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    let channel_config = [ChannelConfig::single_ended(p.PIO0_5)];
    let mut adc = Adc::new(p.ADC0, Irqs, Config::default(), channel_config);

    loop {
        let mut data: [i16; 1] = [0; 1];
        adc.sample(&mut data).await;

        info!("ADC sample = {:#x}", data);

        embassy_imxrt_examples::delay(50000);
    }
}
