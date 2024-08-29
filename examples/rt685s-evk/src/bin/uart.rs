#![no_std]
#![no_main]

use crate::pac::flexcomm0;
use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::flexcomm::Config as FcConfig;
use embassy_imxrt::flexcomm::Flexcomm;
use embassy_imxrt::uart::Config;
use embassy_imxrt::uart::Uart;
use mimxrt685s_pac as pac;

pub use pac::flexcomm0::pselid::Lock as FlexcommLock;
pub use pac::flexcomm0::pselid::Persel as FcFunction;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    let mut fc0_config = FcConfig::default();
    fc0_config.function = FcFunction::Usart;
    fc0_config.lock = FlexcommLock::Unlocked;

    let fc0 = <dyn Flexcomm>::Flexcomm0.new(&fc0_config);
    fc0.enable(&fc0_config);

    let config = Config::default();
    let uart = Uart::new(p.UART, p.UART_CLK, p.UART_TX, p.UART_RX, None, None, config).unwrap();
}
