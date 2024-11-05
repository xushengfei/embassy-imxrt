#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_imxrt::dma::transfer::TransferOptions;
use embassy_imxrt::dma::Dma;
use embassy_imxrt::peripherals::*;
use {defmt_rtt as _, panic_probe as _};

const MAX_BUFFER_LEN: usize = 10;

macro_rules! test_dma_channel {
    ($peripherals: expr, $instance: ident, $number: expr) => {
        let ch = Dma::reserve_channel::<$instance>($peripherals.$instance);
        let mut srcbuf = [0u8; MAX_BUFFER_LEN];
        let mut dstbuf = [0u8; MAX_BUFFER_LEN];

        for idx in 1..MAX_BUFFER_LEN - 1 {
            srcbuf[0..MAX_BUFFER_LEN].fill(0xFF);
            dstbuf[0..MAX_BUFFER_LEN].fill(0xFF);

            srcbuf[0..idx].fill(0xAA);

            ch.write_to_memory(&srcbuf[..], &mut dstbuf[..], TransferOptions::default())
                .await;

            if srcbuf == dstbuf {
                info!("Successfully transferred {} bytes on DMA channel {}", idx, $number,);
            } else {
                error!("Failed to transfer {} bytes on DMA channel {}!", idx, $number);
                panic!("DMA transfer failed");
            }
        }
    };
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Test memory-to-memory DMA transfers");

    test_dma_channel!(p, DMA0_CH0, 0);
    test_dma_channel!(p, DMA0_CH1, 1);
    test_dma_channel!(p, DMA0_CH2, 2);
    test_dma_channel!(p, DMA0_CH3, 3);
    test_dma_channel!(p, DMA0_CH4, 4);
    test_dma_channel!(p, DMA0_CH5, 5);
    test_dma_channel!(p, DMA0_CH6, 6);
    test_dma_channel!(p, DMA0_CH7, 7);
    test_dma_channel!(p, DMA0_CH8, 8);
    test_dma_channel!(p, DMA0_CH9, 9);
    test_dma_channel!(p, DMA0_CH10, 10);
    test_dma_channel!(p, DMA0_CH11, 11);
    test_dma_channel!(p, DMA0_CH12, 12);
    test_dma_channel!(p, DMA0_CH13, 13);
    test_dma_channel!(p, DMA0_CH14, 14);
    test_dma_channel!(p, DMA0_CH15, 15);
    test_dma_channel!(p, DMA0_CH16, 16);
    test_dma_channel!(p, DMA0_CH17, 17);
    test_dma_channel!(p, DMA0_CH18, 18);
    test_dma_channel!(p, DMA0_CH19, 19);
    test_dma_channel!(p, DMA0_CH20, 20);
    test_dma_channel!(p, DMA0_CH21, 21);
    test_dma_channel!(p, DMA0_CH22, 22);
    test_dma_channel!(p, DMA0_CH23, 23);
    test_dma_channel!(p, DMA0_CH24, 24);
    test_dma_channel!(p, DMA0_CH25, 25);
    test_dma_channel!(p, DMA0_CH26, 26);
    test_dma_channel!(p, DMA0_CH27, 27);
    test_dma_channel!(p, DMA0_CH28, 28);
    test_dma_channel!(p, DMA0_CH29, 29);
    test_dma_channel!(p, DMA0_CH30, 30);
    test_dma_channel!(p, DMA0_CH31, 31);

    info!("DMA transfer tests completed");
}
