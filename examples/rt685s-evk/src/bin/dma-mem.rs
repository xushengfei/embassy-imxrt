#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::dma::{transfer::TransferOptions, transfer::Width, Dma};
use {defmt_rtt as _, panic_probe as _};

static SRC_ARRAY1: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
static SRC_ARRAY2: [u8; 10] = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
static mut DST_ARRAY: [u8; 10] = [0; 10];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("DMA memory-to-memory transfers");

    // Reserve DMA channels
    let mut ch1 = Dma::reserve_channel(p.DMA0_CH0);
    let mut ch2 = Dma::reserve_channel(p.DMA0_CH31);

    // Default transfer width is 32 bits
    let mut options = TransferOptions::default();
    options.width = Width::Bit8;

    // SAFETY: use of a mutable static is unsafe
    ch1.write_mem(&SRC_ARRAY1[..], unsafe { &mut DST_ARRAY[..] }, &options);

    //while ch.is_channel_active(0).unwrap() {}
    embassy_imxrt_examples::delay(5_000);

    unsafe {
        if SRC_ARRAY1 == DST_ARRAY {
            info!("DMA transfer #1 completed successfully")
        } else {
            info!("DMA transfer #1 failed")
        }
    }
    // SAFETY: use of a mutable static is unsafe
    ch2.write_mem(&SRC_ARRAY2[..], unsafe { &mut DST_ARRAY[..] }, &options);

    //while ch.is_channel_active(0).unwrap() {}
    embassy_imxrt_examples::delay(10_000);

    unsafe {
        if SRC_ARRAY2 == DST_ARRAY {
            info!("DMA transfer #2 completed successfully")
        } else {
            info!("DMA transfer #2 failed")
        }
    }
}
