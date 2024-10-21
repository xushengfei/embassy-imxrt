#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::crc::{Config, Crc, Polynomial};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing CRC");

    let config = Config::new(Polynomial::Crc32, false, false, false, false, 0x00000000);
    let mut crc = Crc::new(p.CRC, config);

    let output = crc.feed_bytes(b"Embassy") ^ 0xffffffff;

    defmt::assert_eq!(output, 0xebfebe9a);

    cortex_m::asm::bkpt();
}
