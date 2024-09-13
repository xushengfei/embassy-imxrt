#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_imxrt::i2c::{self, I2cSlaveBlocking};
use embassy_imxrt::iopctl::Pull;
use embassy_imxrt::pac;

const SLAVE_ADDR: Option<i2c::Address> = i2c::Address::new(0x20);

fn wait_for_magic(i2c: &impl I2cSlaveBlocking) {
    info!("i2cs example - waiting for ping");

    match i2c.block_until_addressed() {
        Ok(_) => info!("i2cs example - ping successfully received!"),
        Err(e) => error!("i2cs example - ping exited with {:?}", e),
    }

    info!("i2cs example - wait for magic code 0xDEADBEEF");
    let magic_code = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut received: [u8; 4] = [0; 4];

    match i2c.read(&mut received) {
        Ok(_) => {
            info!("i2cs example - read(4) success! Checking Code");

            for (rx, mg) in (&received).iter().zip(magic_code.iter()) {
                if rx != mg {
                    error!("Mismatch got {:?} but expected {:?}", rx, mg);
                }
            }
        }
        Err(e) => error!("i2cs example - read(4) exited with {:?}", e),
    }

    info!("i2cs example - writing back magic code to host");

    match i2c.write(&magic_code) {
        Ok(_) => info!("i2c example - write(4) successfully received!"),
        Err(e) => error!("i2c example - write(4) exited with {:?}", e),
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // NOTE: tested with a raspberry pi 5 as master controller. Commands used:
    // $ i2cdetect -y 1
    // $ i2ctransfer -y 1 w4@0x20 0xDE 0xAD 0xBE 0xEF r4

    let pac = pac::Peripherals::take().unwrap();

    // Ensure SFRO Clock is set to run (power down is cleared)
    pac.sysctl0.pdruncfg0_clr().write(|w| w.sfro_pd().set_bit());

    info!("i2cs example - embassy_imxrt::init");
    let p = embassy_imxrt::init(Default::default());

    info!("i2cs example - I2c::new");
    let i2c = i2c::I2cSlave::new(p.FLEXCOMM2, p.PIO0_18, p.PIO0_17, Pull::Down, SLAVE_ADDR.unwrap()).unwrap();

    embassy_imxrt_examples::delay(500);

    loop {
        wait_for_magic(&i2c);
    }
}
