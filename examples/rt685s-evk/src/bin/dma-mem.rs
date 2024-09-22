#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::dma::Dma;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Configuring DMA memory-to-memory transfer");
    let mut _dma = Dma::new(p.DMA0);
}
