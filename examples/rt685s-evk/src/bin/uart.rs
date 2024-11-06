#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::peripherals::{FLEXCOMM2, FLEXCOMM4};
use embassy_imxrt::uart::Uart;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn usart4_task(uart: Uart<'static, FLEXCOMM4>) {
    info!("RX Task");

    loop {
        let mut buf = [0; 8];

        Timer::after_millis(10).await;

        uart.read_blocking(&mut buf).unwrap();

        let s = core::str::from_utf8(&buf).unwrap();

        info!("Received '{}'", s);
    }
}

#[embassy_executor::task]
async fn usart2_task(uart: Uart<'static, FLEXCOMM2>) {
    info!("TX Task");

    loop {
        let buf = "Testing\0".as_bytes();

        uart.write_blocking(&buf).unwrap();

        Timer::after_millis(10).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("UART test start");

    let usart4 = Uart::new(
        p.FLEXCOMM4,
        p.PIO0_29,
        p.PIO0_30,
        Default::default(),
        Default::default(),
    )
    .unwrap();
    spawner.must_spawn(usart4_task(usart4));

    let usart2 = Uart::new_tx_only(p.FLEXCOMM2, p.PIO0_15, Default::default(), Default::default()).unwrap();
    spawner.must_spawn(usart2_task(usart2));
}
