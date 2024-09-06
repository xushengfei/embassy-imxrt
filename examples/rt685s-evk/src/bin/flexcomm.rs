#![no_std]
#![no_main]

use crate::pac::flexcomm0;
use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::flexcomm::Config as FcConfig;
use embassy_imxrt::flexcomm::Flexcomm;
use mimxrt685s_pac as pac;

pub use pac::clkctl1::flexcomm::fcfclksel::Sel as FcClksel;
pub use pac::flexcomm0::pselid::Lock as FcLock;
pub use pac::flexcomm0::pselid::Persel as FcFunction;

use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    let mut fc0_config = FcConfig::default();
    fc0_config.function = FcFunction::Usart;
    fc0_config.lock = FcLock::Unlocked;
    fc0_config.clksel = FcClksel::SfroClk;

    let fc0 = Flexcomm::new(p.FLEXCOMM0, fc0_config);
    fc0.enable();

    let mut fc1_config = FcConfig::default();
    fc1_config.function = FcFunction::Spi;
    fc1_config.lock = FcLock::Unlocked;
    fc1_config.clksel = FcClksel::FfroClk;

    let fc1 = Flexcomm::new(p.FLEXCOMM1, fc1_config);
    fc1.enable();

    let mut fc2_config = FcConfig::default();
    fc2_config.function = FcFunction::I2c;
    fc2_config.lock = FcLock::Unlocked;
    fc2_config.clksel = FcClksel::AudioPllClk;

    let fc2 = Flexcomm::new(p.FLEXCOMM2, fc2_config);
    fc2.enable();

    let mut fc14_config = FcConfig::default();
    fc14_config.function = FcFunction::Spi;
    fc14_config.lock = FcLock::Unlocked;
    fc14_config.clksel = FcClksel::FfroClk;

    let fc14 = Flexcomm::new(p.FLEXCOMM14, fc14_config);
    fc14.enable();

    let mut fc15_config = FcConfig::default();
    fc15_config.function = FcFunction::I2c;
    fc15_config.lock = FcLock::Unlocked;
    fc15_config.clksel = FcClksel::SfroClk;

    let fc15 = Flexcomm::new(p.FLEXCOMM15, fc15_config);
    fc15.enable();

    fc0.disable();
    fc1.disable();
    fc2.disable();

    fc14.disable();
    fc15.disable();
}
