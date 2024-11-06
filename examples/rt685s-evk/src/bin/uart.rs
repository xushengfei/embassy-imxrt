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
        let mut buf = [0; 5];
        let len = buf.len() as u32;

        uart.read_blocking(&mut buf, len).unwrap();

        info!("Received {:?}", buf);
    }
}

#[embassy_executor::task]
async fn usart2_task(uart: Uart<'static, FLEXCOMM2>) {
    info!("TX Task");

    loop {
        let mut buf = [74, 70, 71, 72, 73];
        let len = buf.len() as u32;

        uart.write_blocking(&mut buf, len).unwrap();

        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    board_init_sfro_clocks();

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

    Timer::after_millis(1000).await;

    let usart2 = Uart::new_tx_only(p.FLEXCOMM2, p.PIO0_15, Default::default(), Default::default()).unwrap();
    spawner.must_spawn(usart2_task(usart2));
}

fn board_init_sfro_clocks() {
    let pac = embassy_imxrt::pac::Peripherals::take().unwrap();

    // Ensure SFRO Clock is set to run (power down is cleared)
    pac.sysctl0.pdruncfg0_clr().write(|w| w.sfro_pd().set_bit());

    info!("Enabling GPIO1 clock");
    pac.clkctl1.pscctl1_set().write(|w| w.hsgpio0_clk_set().set_clock());
    pac.clkctl1.pscctl1_set().write(|w| w.hsgpio1_clk_set().set_clock());

    // Take GPIO0 out of reset
    info!("Clearing GPIO1 reset");
    pac.rstctl1.prstctl1_clr().write(|w| w.hsgpio0_rst_clr().clr_reset());
    pac.rstctl1.prstctl1_clr().write(|w| w.hsgpio1_rst_clr().clr_reset());
}
