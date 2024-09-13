#![no_std]
#![no_main]

use crate::pac::flexcomm0;
use defmt::info;
use defmt_rtt as _;
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
    info!("fc0 enabled");

    let mut fc1_config = FcConfig::default();
    fc1_config.function = FcFunction::Spi;
    fc1_config.lock = FcLock::Locked;
    fc1_config.clksel = FcClksel::FfroClk;
    let fc1 = Flexcomm::new(p.FLEXCOMM1, fc1_config);
    fc1.enable();
    info!("fc1 enabled");

    let mut fc2_config = FcConfig::default();
    fc2_config.function = FcFunction::I2c;
    fc2_config.lock = FcLock::Unlocked;
    fc2_config.clksel = FcClksel::AudioPllClk;
    let fc2 = Flexcomm::new(p.FLEXCOMM2, fc2_config);
    fc2.enable();
    info!("fc2 enabled");

    let mut fc3_config = FcConfig::default();
    fc3_config.function = FcFunction::I2sReceive;
    fc3_config.lock = FcLock::Locked;
    fc3_config.clksel = FcClksel::SfroClk;
    let fc3 = Flexcomm::new(p.FLEXCOMM3, fc3_config);
    fc3.enable();
    info!("fc3 enabled");

    let mut fc4_config = FcConfig::default();
    fc4_config.function = FcFunction::I2sTransmit;
    fc4_config.lock = FcLock::Unlocked;
    fc4_config.clksel = FcClksel::FfroClk;
    let fc4 = Flexcomm::new(p.FLEXCOMM4, fc4_config);
    fc4.enable();
    info!("fc4 enabled");

    let mut fc5_config = FcConfig::default();
    fc5_config.function = FcFunction::Usart;
    fc5_config.lock = FcLock::Locked;
    fc5_config.clksel = FcClksel::AudioPllClk;
    let fc5 = Flexcomm::new(p.FLEXCOMM5, fc5_config);
    fc5.enable();
    info!("fc5 enabled");

    let mut fc6_config = FcConfig::default();
    fc6_config.function = FcFunction::Spi;
    fc6_config.lock = FcLock::Unlocked;
    fc6_config.clksel = FcClksel::FfroClk;
    let fc6 = Flexcomm::new(p.FLEXCOMM6, fc6_config);
    fc6.enable();
    info!("fc6 enabled");

    let mut fc7_config = FcConfig::default();
    fc7_config.function = FcFunction::I2c;
    fc7_config.lock = FcLock::Locked;
    fc7_config.clksel = FcClksel::AudioPllClk;
    let fc7 = Flexcomm::new(p.FLEXCOMM7, fc7_config);
    fc7.enable();
    info!("fc7 enabled");

    let mut fc14_config = FcConfig::default();
    fc14_config.function = FcFunction::Spi;
    fc14_config.lock = FcLock::Unlocked;
    fc14_config.clksel = FcClksel::MasterClk;
    let fc14 = Flexcomm::new(p.FLEXCOMM14, fc14_config);
    fc14.enable();
    info!("fc14 enabled");

    let mut fc15_config = FcConfig::default();
    fc15_config.function = FcFunction::I2c;
    fc15_config.lock = FcLock::Locked;
    fc15_config.clksel = FcClksel::SfroClk;
    let fc15 = Flexcomm::new(p.FLEXCOMM15, fc15_config);
    fc15.enable();
    info!("fc15 enabled");

    fc0.disable();
    info!("fc0 disabled");
    fc1.disable();
    info!("fc1 disabled");
    fc2.disable();
    info!("fc2 disabled");
    fc3.disable();
    info!("fc3 disabled");
    fc4.disable();
    info!("fc4 disabled");
    fc5.disable();
    info!("fc5 disabled");
    fc6.disable();
    info!("fc6 disabled");
    fc7.disable();
    info!("fc7 disabled");

    fc14.disable();
    info!("fc14 disabled");
    fc15.disable();
    info!("fc15 disabled");
}
