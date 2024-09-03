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

        let mut usart = UartRxTx::new();
        usart.init();

        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        //let data = [0xC0, 0xDE];
        //usart.blocking_write(&data).unwrap();
        let mut data = [0xC0, 0xDE];
        usart.write_blocking(&mut data, 2);

        let mut buf = [0; 2];
        //usart.blocking_read(&mut buf).unwrap();
        usart.read_blocking(&mut buf, 2);
        assert_eq!(buf, data);

        usart.deinit();
    }
}
