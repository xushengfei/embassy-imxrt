#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::dma::{transfer::TransferOptions, Dma};
use embassy_imxrt::rng;
use embassy_imxrt::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

macro_rules! test_dma_channel {
    ($peripherals: expr, $rng: expr, $instance: ident, $number: expr) => {
        let ch = Dma::reserve_channel($peripherals.$instance);
        let mut srcbuf = [0u8; 10];
        let mut dstbuf = [1u8; 10];

        // Test the same channel multiple times.
        for idx in 1..4 {
            unwrap!($rng.async_fill_bytes(&mut srcbuf).await);
            srcbuf[0] = idx;
            srcbuf[1] = $number;

            ch.write_to_memory(&srcbuf[..], &mut dstbuf[..], TransferOptions::default())
                .await;

            if srcbuf == dstbuf {
                info!(
                    "DMA transfer {} on channel {} completed successfully: {:02x}",
                    idx,
                    $number,
                    dstbuf.iter().as_slice()
                );
            } else {
                info!("DMA transfer {} on channel {} failed!", idx, $number);
            }
        }
    };
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    let mut rng = rng::Rng::new(p.RNG, Irqs);

    info!("Test memory-to-memory DMA transfers");

    test_dma_channel!(p, rng, DMA0_CH0, 0);
    test_dma_channel!(p, rng, DMA0_CH1, 1);
    test_dma_channel!(p, rng, DMA0_CH2, 2);
    test_dma_channel!(p, rng, DMA0_CH3, 3);
    test_dma_channel!(p, rng, DMA0_CH4, 4);
    test_dma_channel!(p, rng, DMA0_CH5, 5);
    test_dma_channel!(p, rng, DMA0_CH6, 6);
    test_dma_channel!(p, rng, DMA0_CH7, 7);
    test_dma_channel!(p, rng, DMA0_CH8, 8);
    test_dma_channel!(p, rng, DMA0_CH9, 9);
    test_dma_channel!(p, rng, DMA0_CH10, 10);
    test_dma_channel!(p, rng, DMA0_CH11, 11);
    test_dma_channel!(p, rng, DMA0_CH12, 12);
    test_dma_channel!(p, rng, DMA0_CH13, 13);
    test_dma_channel!(p, rng, DMA0_CH14, 14);
    test_dma_channel!(p, rng, DMA0_CH15, 15);
    test_dma_channel!(p, rng, DMA0_CH16, 16);
    test_dma_channel!(p, rng, DMA0_CH17, 17);
    test_dma_channel!(p, rng, DMA0_CH18, 18);
    test_dma_channel!(p, rng, DMA0_CH19, 19);
    test_dma_channel!(p, rng, DMA0_CH20, 20);
    test_dma_channel!(p, rng, DMA0_CH21, 21);
    test_dma_channel!(p, rng, DMA0_CH22, 22);
    test_dma_channel!(p, rng, DMA0_CH23, 23);
    test_dma_channel!(p, rng, DMA0_CH24, 24);
    test_dma_channel!(p, rng, DMA0_CH25, 25);
    test_dma_channel!(p, rng, DMA0_CH26, 26);
    test_dma_channel!(p, rng, DMA0_CH27, 27);
    test_dma_channel!(p, rng, DMA0_CH28, 28);
    test_dma_channel!(p, rng, DMA0_CH29, 29);
    test_dma_channel!(p, rng, DMA0_CH30, 30);
    test_dma_channel!(p, rng, DMA0_CH31, 31);

    info!("DMA transfer tests completed");
}
