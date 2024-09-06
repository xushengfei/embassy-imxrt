#![no_std]
#![no_main]

use crate::pac::flexcomm0;
use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::flexcomm::Config as FcConfig;
use embassy_imxrt::flexcomm::Flexcomm;
use mimxrt685s_pac as pac;

pub use pac::flexcomm0::pselid::Lock as FcLock;
pub use pac::flexcomm0::pselid::Persel as FcFunction;

use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    let mut fc0_config = FcConfig::default();
    fc0_config.function = FcFunction::Usart;
    fc0_config.lock = FcLock::Unlocked;

    let fc0 = Flexcomm::new(p.FLEXCOMM0, fc0_config);
    fc0.enable();

    let mut fc1_config = FcConfig::default();
    fc1_config.function = FcFunction::Spi;
    fc1_config.lock = FcLock::Unlocked;

    let fc1 = Flexcomm::new(p.FLEXCOMM1, fc1_config);
    fc1.enable();

    let mut fc2_config = FcConfig::default();
    fc2_config.function = FcFunction::I2c;
    fc2_config.lock = FcLock::Unlocked;

    let fc2 = Flexcomm::new(p.FLEXCOMM2, fc2_config);
    fc2.enable();

    fc0.disable();
    fc1.disable();
    fc2.disable();
}
