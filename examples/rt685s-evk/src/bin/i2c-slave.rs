#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_imxrt::i2c::{self, I2cSlaveBlocking};
use embassy_imxrt::iopctl::Pull;
use embassy_imxrt::pac;
use embassy_time::Timer;

const SLAVE_ADDR: Option<i2c::Address> = i2c::Address::new(0x20);

fn slave_service(i2c: &impl I2cSlaveBlocking) {
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let pac = pac::Peripherals::take().unwrap();

    // Ensure SFRO Clock is set to run (power down is cleared)
    pac.sysctl0.pdruncfg0_clr().write(|w| w.sfro_pd().set_bit());

    info!("i2cs example - embassy_imxrt::init");
    let p = embassy_imxrt::init(Default::default());

    // NOTE: Tested with a raspberry pi 5 as master controller connected FC2 to i2c on Pi5
    //       Test program here: https://github.com/jerrysxie/pi5-i2c-test
    info!("i2cs example - I2c::new");
    let i2c = i2c::I2cSlave::new_blocking(
        p.FLEXCOMM2,
        p.PIO0_18,
        p.PIO0_17,
        Pull::Down,
        SLAVE_ADDR.unwrap(),
        p.DMA0_CH4,
    )
    .unwrap();

    loop {
        slave_service(&i2c);
        Timer::after_millis(1000).await;
    }
}
