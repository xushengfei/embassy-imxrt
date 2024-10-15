//! This example shows how to use SPI (Serial Peripheral Interface) in the RT632.
//!
//!

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::spi::Spi;
use {defmt_rtt as _, embassy_imxrt as _, mimxrt685s_pac as pac, panic_probe as _};
//use embassy_imxrt::{gpio, spi};
//use gpio::{Level, Output};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    /*
    info!("Hello World!");

        // configure gpio pins

        let miso = p.;
        let mosi = p.;
        let clk = p.;
        let spi_cs = p.;
    */

    // create SPI
    let mut config = spi::Config::default();
    config.frequency = 2_000_000;
    let mut spi = Spi::new_blocking(p.SPI0, clk, mosi, miso, config);
    /*

        // Configure CS
        let mut cs = Output::new(spi_cs, Level::Low);

        loop {
            cs.set_low();
            let mut buf = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
            spi.blocking_transfer_in_place(&mut buf).unwrap();
            cs.set_high();

            info!("spidata: {}", buf);
        }
    */
}
