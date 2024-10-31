#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_imxrt::i2c::{self, Async, I2cMaster, I2cMasterAsync, I2cSlave, I2cSlaveAsync};
use embassy_imxrt::iopctl::Pull;
use embassy_imxrt::peripherals::{DMA0_CH4, DMA0_CH9, FLEXCOMM2, FLEXCOMM4};
use embassy_time::Timer;

const SLAVE_ADDR: Option<i2c::Address> = i2c::Address::new(0x20);

#[embassy_executor::task]
async fn slave_service(mut slave: I2cSlave<'static, FLEXCOMM2, Async, DMA0_CH4>) {
    loop {
        let magic_code = [0xF0, 0x05, 0xBA, 0x11];

        let mut cmd_length: [u8; 1] = [0xAA; 1];
        info!("i2cs example - wait for cmd - read cmd length first");
        slave.listen(&mut cmd_length, false).await.unwrap();
        info!("i2cs cmd length = {:02X}", cmd_length);

        let mut cmd: [u8; 4] = [0xAA; 4];
        info!("i2cs example - wait for cmd - read the actual cmd");
        slave.listen(&mut cmd, true).await.unwrap();
        info!("i2cs cmd length = {:02X}", cmd_length);

        if cmd == [0xDE, 0xAD, 0xBE, 0xEF] {
            info!("i2cs example - receive init cmd");
        } else if cmd == [0xDE, 0xCA, 0xFB, 0xAD] {
            info!("i2cs example - receive magic cmd, writing back magic code to host");
            slave.respond(&magic_code).await.unwrap();
        } else {
            error!("i2cs unexpected cmd = {:02X}", cmd);
            panic!("i2cs example - unexpected cmd");
        }

        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn master_service(mut master: I2cMaster<'static, FLEXCOMM4, Async, DMA0_CH9>) {
    const ADDR: u8 = 0x20;

    let init_cmd = [0x05, 0xDE, 0xAD, 0xBE, 0xEF];
    let magic_cmd = [0x05, 0xDE, 0xCA, 0xFB, 0xAD];

    let mut i: u32 = 0;
    loop {
        if i % 2 == 0 {
            info!("i2cm write init cmd");
            master.write(ADDR, &init_cmd).await.unwrap();
        } else {
            let mut code: [u8; 4] = [0xAA; 4];
            info!("i2cm write magic code");
            master.write_read(ADDR, &magic_cmd, &mut code).await.unwrap();
            info!("i2cm magic code = {:02x}", code);
        }
        i += 1;
        Timer::after_secs(2).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("i2c loopback example");
    let p = embassy_imxrt::init(Default::default());

    let slave = i2c::I2cSlave::new_async(
        p.FLEXCOMM2,
        p.PIO0_18,
        p.PIO0_17,
        Pull::None,
        SLAVE_ADDR.unwrap(),
        p.DMA0_CH4,
    )
    .unwrap();

    let master = i2c::I2cMaster::new_async(
        p.FLEXCOMM4,
        p.PIO0_29,
        p.PIO0_30,
        Pull::Up,
        i2c::Speed::Standard,
        i2c::TimeoutSettings {
            hw_timeout: true,
            sw_timeout: embassy_time::Duration::from_millis(10000),
        },
        p.DMA0_CH9,
    )
    .await
    .unwrap();

    spawner.must_spawn(master_service(master));
    spawner.must_spawn(slave_service(slave));
}
