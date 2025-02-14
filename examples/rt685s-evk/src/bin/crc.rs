#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::crc::{Config, Crc};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing CRC");

    let mut crc = Crc::new(p.CRC, Default::default());
    let output = crc.feed_bytes(b"123456789");
    defmt::assert_eq!(output, 0x29b1);

    cortex_m::asm::bkpt();
}
