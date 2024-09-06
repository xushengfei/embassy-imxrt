#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::uart::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_imxrt::init(Default::default());

    {
        //let config = Config::default();
        //let mut usart = Uart::new_blocking(&mut usart, &mut rx, &mut tx, config).unwrap(); //toto change

        info!("UART test start");
        let mut usart = UartRxTx::new();
        usart.init();

        info!("UART test init() done");
        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        //let data = [0xC0, 0xDE];
        //usart.blocking_write(&data).unwrap();

        let mut status = GenericStatus::Success;
        let mut data = [0xC0, 0xDE];
        status = usart.write_blocking(&mut data, 2);
        if status != GenericStatus::Success {
            info!("UART test write_blocking() failed");
        } else {
            info!("UART test write_blocking() done");
        }

        let mut buf = [0; 2];
        //usart.blocking_read(&mut buf).unwrap();
        status = usart.read_blocking(&mut buf, 2);
        if status != GenericStatus::Success {
            info!("UART test read_blocking() failed");
        } else {
            info!("UART test read_blocking() done");
        }

        if status == GenericStatus::Success {
            //assert_eq!(buf, data);
            if &buf[0..2] == &data[..] {
                info!("UART test passed");
            } else {
                info!("UART test failed");
            }
        } else {
            info!("UART test failed");
        }

        usart.deinit();
        info!("UART test deinit() done");
        info!("UART test done");

        embassy_imxrt_examples::delay(50000);
    }
}
