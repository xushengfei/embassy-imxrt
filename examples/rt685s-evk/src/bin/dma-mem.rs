#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::dma::Dma;
use {defmt_rtt as _, panic_probe as _};

static ARRAY1: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
static mut ARRAY2: [u8; 10] = [0; 10];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("DMA memory-to-memory transfer");
    let mut dma = Dma::new(p.DMA0);

    // SAFETY: use of a mutable static is unsafe
    match dma.configure_channel(0, &ARRAY1[..], unsafe { &mut ARRAY2[..] }) {
        Ok(v) => v,
        Err(_e) => info!("failed to configure DMA channel"),
    };

    match dma.enable_channel(0) {
        Ok(v) => v,
        Err(_e) => info!("failed to enable DMA channel"),
    };

    match dma.trigger_channel(0) {
        Ok(v) => v,
        Err(_e) => info!("failed to trigger DMA channel"),
    };

    while dma.is_channel_active(0).unwrap() {}

    unsafe {
        if ARRAY1 == ARRAY2 {
            info!("DMA transfer completed successfully")
        } else {
            info!("DMA transfer failed")
        }
    }
}
