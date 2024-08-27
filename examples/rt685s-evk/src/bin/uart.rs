#![no_std]
#![no_main]

use crate::pac::flexcomm0;
use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::flexcomm::Flexcomm;
use embassy_imxrt::uart::Config;
use embassy_imxrt::uart::Uart;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    let mut fc0_config = flexcomm::Config::default();
    fc0_config.function = flexcomm::Function::Usart;
    fc0_config.lock = flexcomm::FlexcommLock::Unlocked;

    flexcomm::Flexcomm0.enable(&fc0_config);

    let config = Config::default();
    let uart = Uart::new(p.UART, p.UART_CLK, p.UART_TX, p.UART_RX, None, None, config).unwrap();
}
