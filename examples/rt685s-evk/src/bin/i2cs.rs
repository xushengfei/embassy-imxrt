#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU8, Ordering};

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::i2c::{self, Address, SlaveBlocking};
use embassy_imxrt::{self, iopctl};
use embassy_sync::once_lock::OnceLock;

const SLV_ADDR: Option<Address> = Address::new(0x10);
const MAGIC_SYNC: u8 = 0xAA;

enum Command {
    Read(u8),
    Write(u8, u8),
}

impl Command {
    fn new(raw: [u8; 4]) -> Option<Self> {
        match raw[3] {
            MAGIC_SYNC => match raw[0] {
                0 => Some(Command::Read(raw[1])),
                1 => Some(Command::Write(raw[1], raw[2])),
                _ => None,
            },
            _ => None,
        }
    }
}

static INTERNAL_REGISTERS: OnceLock<[AtomicU8; 256]> = OnceLock::new();

async fn blocking_interface(i2cs: &impl i2c::SlaveBlocking) {
    info!("Waiting for pings...");
    while i2cs.wait_for_ping().is_err() {
        info!("Incorrect ping... waiting again");
    }

    info!("Awaiting command from host controller...");

    // receive command header
    let four_bytes = i2cs.blocking_read::<4>();
    match four_bytes {
        Ok(data) => {
            if let Some(cmd) = Command::new(data) {
                match cmd {
                    Command::Read(addr) => {
                        info!("Read: {:?}", addr);
                        let val = INTERNAL_REGISTERS.get().await[addr as usize].load(Ordering::Relaxed);
                        let rsp = [val, MAGIC_SYNC];
                        match i2cs.blocking_write(&rsp) {
                            Ok(_) => info!("Success!"),
                            Err(_) => info!("I2c bus error encountered."),
                        }
                    }
                    Command::Write(addr, value) => {
                        info!("Write: {:?} = {:?}", addr, value);
                        INTERNAL_REGISTERS.get().await[addr as usize].store(value, Ordering::Relaxed);

                        let rsp = [MAGIC_SYNC];
                        match i2cs.blocking_write(&rsp) {
                            Ok(_) => info!("Success!"),
                            Err(_) => info!("I2c bus error encountered."),
                        }
                    }
                }
            } else {
                info!("Command structure error!");
            }
        }
        Err(_) => info!("I2C Bus Error!"),
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    embassy_imxrt::gpio::Port::init(embassy_imxrt::gpio::Port::Port0);

    // configure pins SCL+SDA for FCn:
    // remember to set J12 to 3.3v to match pull-ups on rpi5!
    // func=1
    let sda = &p.PIO0_17;
    let scl = &p.PIO0_18;

    use embassy_imxrt::iopctl::IopctlPin;
    sda.set_function(iopctl::Function::F1)
        .set_pull(iopctl::Pull::None)
        .disable_input_buffer()
        .set_slew_rate(iopctl::SlewRate::Standard)
        .set_drive_strength(iopctl::DriveStrength::Normal)
        .disable_analog_multiplex()
        .set_drive_mode(iopctl::DriveMode::OpenDrain)
        .set_input_polarity(iopctl::Polarity::ActiveHigh);
    scl.set_function(iopctl::Function::F1)
        .set_pull(iopctl::Pull::None)
        .disable_input_buffer()
        .set_slew_rate(iopctl::SlewRate::Standard)
        .set_drive_strength(iopctl::DriveStrength::Normal)
        .disable_analog_multiplex()
        .set_drive_mode(iopctl::DriveMode::OpenDrain)
        .set_input_polarity(iopctl::Polarity::ActiveHigh);

    info!("iMXRT685 EVK peripherals initialized");

    INTERNAL_REGISTERS.get_or_init(|| [const { AtomicU8::new(0) }; 256]);

    let i2cs = i2c::Slave::new(p.FLEXCOMM2, SLV_ADDR.unwrap()).unwrap();

    info!("I2C Slave device configured for address {:?}", u8::from(i2cs.address()));

    //info!("Awaiting ping from host to initiate sync...");
    //let _ = i2cs.blocking_read::<0>().unwrap();

    embassy_imxrt_examples::delay(5_000);

    loop {
        blocking_interface(&i2cs).await;
    }
}
