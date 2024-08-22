#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
//use embassy_imxrt::uart::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    {
        let config = Config::default();
        let mut usart = Uart::new_blocking(&mut usart, &mut rx, &mut tx, config).unwrap(); //toto change

        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        let data = [0xC0, 0xDE];
        usart.blocking_write(&data).unwrap();

        let mut buf = [0; 2];
        usart.blocking_read(&mut buf).unwrap();
        assert_eq!(buf, data);
    }
}
