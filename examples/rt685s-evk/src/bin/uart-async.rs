#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::uart::{Async, Uart};
use embassy_imxrt::{bind_interrupts, peripherals, uart};
use embassy_time::Timer;

bind_interrupts!(struct Irqs {
    FLEXCOMM2 => uart::InterruptHandler<peripherals::FLEXCOMM2>;
    FLEXCOMM4 => uart::InterruptHandler<peripherals::FLEXCOMM4>;
});

const BUFLEN: usize = 16;

#[embassy_executor::task]
async fn usart4_task(mut uart: Uart<'static, Async>) {
    info!("RX Task");

    loop {
        let mut rx_buf = [0; BUFLEN];
        uart.read(&mut rx_buf).await.unwrap();

        Timer::after_millis(10).await;

        let tx_buf = [0xaa; BUFLEN];
        uart.write(&tx_buf).await.unwrap();
    }
}

#[embassy_executor::task]
async fn usart2_task(mut uart: Uart<'static, Async>) {
    info!("TX Task");

    loop {
        let tx_buf = [0x55; BUFLEN];
        uart.write(&tx_buf).await.unwrap();

        Timer::after_millis(10).await;

        let mut rx_buf = [0x00; BUFLEN];
        uart.read(&mut rx_buf).await.unwrap();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("UART test start");

    let usart4 = Uart::new_with_rtscts(
        p.FLEXCOMM4,
        p.PIO0_29,
        p.PIO0_30,
        p.PIO1_0,
        p.PIO0_31,
        Irqs,
        p.DMA0_CH9,
        p.DMA0_CH8,
        Default::default(),
    )
    .unwrap();
    spawner.must_spawn(usart4_task(usart4));

    let usart2 = Uart::new_with_rtscts(
        p.FLEXCOMM2,
        p.PIO0_15,
        p.PIO0_16,
        p.PIO0_18,
        p.PIO0_17,
        Irqs,
        p.DMA0_CH5,
        p.DMA0_CH4,
        Default::default(),
    )
    .unwrap();
    spawner.must_spawn(usart2_task(usart2));

    #[cfg(feature = "test-parser")]
    test_parser_macros::pass_test();
}
