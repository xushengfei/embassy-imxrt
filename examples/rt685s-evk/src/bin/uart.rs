#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::uart::Uart;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    board_init_sfro_clocks();

    info!("UART test start");

    // Validating read on FC1
    let usart = Uart::new(p.FLEXCOMM1, p.PIO0_8, p.PIO0_9, Default::default(), Default::default()).unwrap();

    // To test read send the data on tera term / putty and verify from the buffer
    let mut buf = [0; 5];

    let result = usart.read_blocking(&mut buf, 5);
    match result {
        Ok(()) => {
            for i in &buf {
                info!("{} ", *i as char);
            }
            info!("UART test read_blocking() done");
        }
        Err(e) => info!("UART test read_blocking() failed, result: {:?}", e),
    }

    let _ = usart.deinit();

    // Validating write on FC2
    let usart = Uart::new_tx_only(p.FLEXCOMM2, p.PIO0_15, Default::default(), Default::default()).unwrap();

    let mut data = [74, 70, 71, 72, 73];
    let result = usart.write_blocking(&mut data, 5);
    match result {
        Ok(()) => info!("UART test write_blocking() done"),
        Err(e) => info!("UART test write_blocking failed, result: {:?}", e),
    }

    let _ = usart.deinit();
    info!("UART test done");

    loop {
        Timer::after_millis(1000).await;
    }
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
