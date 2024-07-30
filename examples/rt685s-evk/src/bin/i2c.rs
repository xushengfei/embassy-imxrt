#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_imxrt::i2c::config::Config;
use embassy_imxrt::i2c::i2cm::{HalI2c, I2c};
use embassy_imxrt::pac;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Link to data sheet for accelerometer on the RT685S-EVK
    // https://www.nxp.com/docs/en/data-sheet/FXOS8700CQ.pdf
    // Max Freq is 400 kHz
    // Address is 0x1E, 0x1D, 0x1C or 0x1F

    // Link to schematics for RT685S-EVK
    // https://www.nxp.com/downloads/en/design-support/RT685-DESIGNFILES.zip
    // File: SPF-35099_E2.pdf
    // Page 10 shows ACC Sensor at I2C address 0x1E

    // Link to RT6xx User Manual
    // https://www.nxp.com/webapp/Download?colCode=UM11147

    // Acc is connected to P0_18_FC2_SCL and P0_17_FC2_SDA for I2C
    // Acc RESET gpio is P1_7_RST
    let pac = pac::Peripherals::take().unwrap();

    info!("i2c example - Configure Pins");
    board_init_pins(&pac);

    info!("i2c example - Configure GPIOs");
    board_init_gpios(&pac);

    info!("i2c example - embassy_imxrt::init");
    let p = embassy_imxrt::init(Default::default());

    info!("i2c example - I2c::new");
    let mut i2c = I2c::new(&pac, p.FLEXCOMM2, Config::default());

    // Read WHO_AM_I register, 0x0D to get value 0xC7 (1100 0111)
    info!("i2c example - ACC WHO_AM_I register check");

    let mut reg = [0u8; 1];
    reg[0] = 0xAA;
    let result = i2c.write_read(0x1E, &[0x0D], &mut reg);
    if result.is_ok() {
        info!("i2c example - Read WHO_AM_I register: {:02X}", reg[0]);
    } else {
        error!("i2c example - Error reading WHO_AM_I register {}", result.unwrap_err());
    }

    // reg[0] = 0xAA;
    // let result = i2c.write_read(0x1E, &[0x00], &mut reg);
    // if result.is_ok() {
    //     info!("i2c example - Read STATUS register: {:02X}", reg[0]);
    // } else {
    //     error!("i2c example - Error reading STATUS register {}", result.unwrap_err());
    // }

    info!("i2c example - Done!  Busy Loop...");
    loop {
        embassy_imxrt_examples::delay(50_000_000);
    }
}

fn board_init_pins(p: &pac::Peripherals) {
    // Ensure SFRO Clock is set to run (power down is cleared)
    p.sysctl0.pdruncfg0_clr().write(|w| w.sfro_pd().set_bit());

    // Configure IO Pad Control 1_7 for ACC Reset Pin
    //
    // Pin is configured as PIO1_7
    // Disable pull-up / pull-down function
    // Enable pull-down function
    // Disable input buffer function
    // Normal mode
    // Normal drive
    // Analog mux is disabled
    // Pseudo Output Drain is disabled
    // Input function is not inverted

    p.iopctl.pio1_7().write(|w| {
        w.fsel()
            .function_0()
            .pupdena()
            .disabled()
            .pupdsel()
            .pull_down()
            .ibena()
            .disabled()
            .slewrate()
            .normal()
            .fulldrive()
            .normal_drive()
            .amena()
            .disabled()
            .odena()
            .disabled()
            .iiena()
            .disabled()
    });

    // Configure IO Pad Control 1_5 for ACC Interrupt Pin
    //
    // Pin is configured as PIO1_5
    // Disable pull-up / pull-down function
    // Enable pull-down function
    // Disable input buffer function
    // Normal mode
    // Normal drive
    // Analog mux is disabled
    // Pseudo Output Drain is disabled
    // Input function is not inverted
    p.iopctl.pio1_5().write(|w| {
        w.fsel()
            .function_0()
            .pupdena()
            .disabled()
            .pupdsel()
            .pull_down()
            .ibena()
            .disabled()
            .slewrate()
            .normal()
            .fulldrive()
            .normal_drive()
            .amena()
            .disabled()
            .odena()
            .disabled()
            .iiena()
            .disabled()
    });
}

fn board_init_gpios(p: &pac::Peripherals) {
    info!("Enabling GPIO1 clock");
    p.clkctl1.pscctl1_set().write(|w| w.hsgpio1_clk_set().set_clock());

    // Take GPIO0 out of reset
    info!("Clearing GPIO1 reset");
    p.rstctl1.prstctl1_clr().write(|w| w.hsgpio1_rst_clr().clr_reset());

    // Set GPIO1_7 (Reset) as ouptut
    info!("Configuring GPIO1_7 as output");
    p.gpio.dirset(1).write(|w| unsafe { w.dirsetp().bits(1 << 7) });

    // Set GPIO1_7 (Reset) as low
    info!("Configuring GPIO1_7 as low");
    p.gpio.set(1).write(|w| unsafe { w.setp().bits(1 << 5) });

    // Set GPIO1_5 (Interrupt) as input
    info!("Configuring GPIO1_5 as input");
    p.gpio.dirclr(1).write(|w| unsafe { w.dirclrp().bits(1 << 5) });
}
