#![no_std]
#![no_main]

use crate::pac::flexcomm0;
use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::flexcomm::Config as FcConfig;
use embassy_imxrt::flexcomm::Flexcomm;
use embassy_imxrt::uart::Config;
use embassy_imxrt::uart::Uart;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    let mut fc0_config = FcConfig::default();
    fc0_config.function = flexcomm::Function::Usart;
    fc0_config.lock = flexcomm::FlexcommLock::Unlocked;

    fc0 = flexcomm::Flexcomm0.new(&fc0_config);
    fc0.enable(&fc0_config);

    let config = Config::default();
    let uart = Uart::new(p.UART, p.UART_CLK, p.UART_TX, p.UART_RX, None, None, config).unwrap();
}
