#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::dma::{util::TransferOptions, ChannelId, Dma};
use {defmt_rtt as _, panic_probe as _};

static ARRAY1: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
static mut ARRAY2: [u8; 10] = [0; 10];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("DMA memory-to-memory transfer");
    let mut dma = Dma::new(p.DMA0);
    let mut ch = dma.reserve_channel(ChannelId::Channel10);

    // SAFETY: use of a mutable static is unsafe
    ch.write_mem(&ARRAY1[..], unsafe { &mut ARRAY2[..] }, TransferOptions::default());

    //while ch.is_channel_active(0).unwrap() {}
    embassy_imxrt_examples::delay(50_000); // TODO

    unsafe {
        if ARRAY1 == ARRAY2 {
            info!("DMA transfer completed successfully")
        } else {
            info!("DMA transfer failed")
        }
    }
}
