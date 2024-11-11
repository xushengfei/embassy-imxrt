#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_imxrt::i2c::{
    slave::{Address, I2cSlave},
    Blocking,
};
use embassy_imxrt::pac;
use embassy_imxrt::peripherals::DMA0_CH4;

const SLAVE_ADDR: Option<Address> = Address::new(0x20);

#[embassy_executor::task]
async fn slave_service(i2c: I2cSlave<'static, Blocking, DMA0_CH4>) {
    loop {
        let magic_code = [0xF0, 0x05, 0xBA, 0x11];

        let mut cmd_length: [u8; 1] = [0xAA; 1];
        info!("i2cs example - wait for cmd - read cmd length first");
        i2c.listen(&mut cmd_length).unwrap();

        let mut cmd: [u8; 4] = [0xAA; 4];
        info!("i2cs example - wait for cmd - read the actual cmd");
        i2c.listen(&mut cmd).unwrap();

        if cmd == [0xDE, 0xAD, 0xBE, 0xEF] {
            info!("i2cs example - receive init cmd");
        } else if cmd == [0xDE, 0xCA, 0xFB, 0xAD] {
            info!("i2cs example - receive magic cmd, writing back magic code to host");
            i2c.respond(&magic_code).unwrap();
        } else {
            error!("unexpected cmd = {:02X}", cmd);
            panic!("i2cs example - unexpected cmd");
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let pac = pac::Peripherals::take().unwrap();

    // Ensure SFRO Clock is set to run (power down is cleared)
    pac.sysctl0.pdruncfg0_clr().write(|w| w.sfro_pd().set_bit());

    info!("i2cs example - embassy_imxrt::init");
    let p = embassy_imxrt::init(Default::default());

    // NOTE: Tested with a raspberry pi 5 as master controller connected FC2 to i2c on Pi5
    //       Test program here: https://github.com/jerrysxie/pi5-i2c-test
    info!("i2cs example - I2c::new");
    let i2c = I2cSlave::new_blocking(p.FLEXCOMM2, p.PIO0_18, p.PIO0_17, SLAVE_ADDR.unwrap(), p.DMA0_CH4).unwrap();

    spawner.must_spawn(slave_service(i2c));
}
