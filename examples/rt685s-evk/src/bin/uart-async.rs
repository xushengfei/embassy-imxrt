#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::{error, info};
use defmt_bbq::DefmtConsumer;
use embassy_executor::Spawner;
use embassy_imxrt::uart::{Async, Config, Uart};
use embassy_imxrt::{bind_interrupts, peripherals, uart};
use embassy_time::Timer;
use {defmt_bbq as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FLEXCOMM2 => uart::InterruptHandler<peripherals::FLEXCOMM2>;
    FLEXCOMM4 => uart::InterruptHandler<peripherals::FLEXCOMM4>;
});

#[embassy_executor::task]
async fn usart4_task(mut uart: Uart<'static, Async>, mut consumer: DefmtConsumer) {
    info!("Message task!!");

    loop {
        if let Ok(grant) = consumer.read() {
            let len = grant.len();

            uart.write(&grant).await.unwrap();
            grant.release(len);

            Timer::after_millis(100).await;
        }
    }
}

#[embassy_executor::task]
async fn logging_task() {
    info!("Starting logging task");
    let mut counter = 0;

    loop {
        counter += 1;

        info!("Counter value: {}", counter);

        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn long_errors() {
    let mut counter = 0;
    let mut buf = [0; 256];

    loop {
        counter += 1;

        buf[0] = (counter & 0xff) as u8;
        buf[1] = ((counter >> 8) & 0xff) as u8;
        buf[2] = ((counter >> 16) & 0xff) as u8;
        buf[3] = ((counter >> 24) & 0xff) as u8;

        error!(
            r##"Lorem ipsum odor amet, consectetuer adipiscing
        elit. Suscipit amet dignissim; proin pharetra quam
        quisque. Eleifend et platea facilisi accumsan mi; lorem
        pretium felis. Suspendisse ridiculus suscipit aenean amet
        fusce integer tristique. Eros tempor leo sociosqu vivamus est
        platea velit curabitur. Suspendisse convallis laoreet elit
        etiam proin quisque accumsan dignissim. Rutrum in himenaeos
        vitae; viverra ridiculus faucibus. Cras elit sociosqu donec
        platea enim; luctus montes. Aliquam suspendisse montes potenti
        varius vitae --> {:?} {:?}"##,
            counter, buf
        );

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let consumer = defmt_bbq::init().unwrap();

    let p = embassy_imxrt::init(Default::default());

    info!("UART test start");

    let config = Config {
        baudrate: 1_000_000,
        ..Default::default()
    };

    let usart4 = Uart::new_with_rtscts(
        p.FLEXCOMM4,
        p.PIO0_29,
        p.PIO0_30,
        p.PIO1_0,
        p.PIO0_31,
        Irqs,
        p.DMA0_CH9,
        p.DMA0_CH8,
        config,
    )
    .unwrap();
    spawner.must_spawn(usart4_task(usart4, consumer));

    spawner.must_spawn(logging_task());
    spawner.must_spawn(long_errors());
}
