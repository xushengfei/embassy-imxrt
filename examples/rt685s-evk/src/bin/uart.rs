#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::uart::*;
use embassy_imxrt::uart_setting::FlexcommFunc;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_imxrt::init(Default::default());

    {
        board_init_sfro_clocks();

        info!("UART test start");
        let mut usart = UartRxTx::new(FlexcommFunc::Flexcomm2);
        usart.init();

        info!("UART init() complete");

        // To test connect an FTDI cable and verify the data received from uart tx on Tera term/ putty
        let mut status = GenericStatus::Success;
        let mut data = [0x55, 0x56, 0x41, 0x42, 0x44]; //[0x41, 0x42, 0x43, 0x44, 0x45];
        status = usart.write_blocking(&mut data, 5);
        if status != GenericStatus::Success {
            info!("UART test write_blocking() failed");
        } else {
            info!("UART test write_blocking() done");
        }

        // To test read send the data on tera term / putty and verify from the buffer
        /*  let mut buf = [0; 5];
        status = usart.read_blocking(&mut buf, 5);
        if status != GenericStatus::Success {
            info!("UART test read_blocking() failed");
        } else {
            info!("UART test read_blocking() done");
        }

        if status == GenericStatus::Success {
            //assert_eq!(buf, data);
            if &buf[0..5] == &data[..] {
                info!("UART test passed");
            } else {
                info!("UART test failed");
            }
        } else {
            info!("UART test failed");
        }*/

        usart.deinit();
        info!("UART test deinit() done");

        info!("UART test done");

        loop {
            embassy_imxrt_examples::delay(50_000_000);
        }
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
