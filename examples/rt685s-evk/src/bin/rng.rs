#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::rng::Rng;
use embassy_imxrt::{bind_interrupts, peripherals, rng};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing RNG");
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut buf = [0u8; 16];

    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}
