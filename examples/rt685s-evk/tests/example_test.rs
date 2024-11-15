#![no_std]
#![no_main]

// #[embedded_test::tests]
#[cfg(test)]
#[embedded_test::tests]
mod tests {
    use defmt::info;
    use defmt_rtt as _;
    use embassy_imxrt;
    use embassy_imxrt::gpio;
    use embassy_imxrt::Peripherals;
    use embassy_time::Timer;

    use embassy_imxrt::gpio::SenseDisabled;
    // Optional: A init function which is called before every test
    // asyncness of init fn is optional
    #[init]
    fn init() -> Peripherals {
        info!("Initializing");
        embassy_imxrt::init(Default::default())
    }

    // // A test which takes the state returned by the init function (optional)
    #[test]
    async fn test_blink(p: Peripherals) {
        unsafe { gpio::init() };

        let mut led = gpio::Output::new(
            p.PIO0_26,
            gpio::Level::Low,
            gpio::DriveMode::PushPull,
            gpio::DriveStrength::Normal,
            gpio::SlewRate::Standard,
        );
        for _ in 1..5 {
            Timer::after_millis(500).await;
            led.toggle();
        }
        info!("It works");
        assert!(true);
    }

    // // // A test which takes the state returned by the init function (optional)
    // #[test]
    // #[should_panic]
    // async fn test_should_panic(p: Peripherals) {
    //     for _ in 1..5 {
    //         Timer::after_millis(500).await;
    //         led.toggle();
    //     }
    //     info!("It works");
    //     assert!(false);
    // }

    // // // A test which takes the state returned by the init function (optional)
    // #[test]
    // async fn test_fast_blink(p: Peripherals) {
    //     for _ in 1..50 {
    //         Timer::after_millis(100).await;
    //         led.toggle();
    //     }
    //     info!("It works");
    //     assert!(true);
    // }

    // // A test which takes the state returned by the init function (optional)
    #[test]
    async fn test_gpio_flex(p: Peripherals) {
        info!("Initializing GPIO");
        unsafe { gpio::init() };

        // Start with a level sensing disabled, output only state
        let flex = gpio::Flex::<SenseDisabled>::new(p.PIO1_0);

        // enable level sensing
        let mut flex = flex.enable_sensing();

        // set pin output bit to high before setting direction
        flex.set_high();

        // set direction as output
        flex.set_as_output(
            gpio::DriveMode::PushPull,
            gpio::DriveStrength::Normal,
            gpio::SlewRate::Standard,
        );

        // check pin level is high
        assert!(flex.is_high());

        // toggle pin
        flex.toggle();

        // check pin level is low
        assert!(flex.is_low());

        // set pin direction to output with reverse polarity
        flex.set_as_input(gpio::Pull::None, gpio::Inverter::Enabled);

        // check pin level is high
        assert!(flex.is_high());

        // set pin output bit to high
        flex.set_high();

        // check pin level is still high
        assert!(flex.is_high());

        // set pin direction to output again
        flex.set_as_output(
            gpio::DriveMode::PushPull,
            gpio::DriveStrength::Normal,
            gpio::SlewRate::Standard,
        );

        // check pin level is now low
        assert!(flex.is_low());

        // disable level sensing
        let mut flex = flex.disable_sensing();

        // set pin level high
        flex.set_high();

        // re-enable level sensing
        let mut flex = flex.enable_sensing();

        // check pin level is high
        assert!(flex.is_high());

        // toggle pin
        flex.toggle();

        // check pin level is low
        assert!(flex.is_low());

        let mut flex = flex.disable_sensing();

        // toggle pin
        flex.toggle();

        let flex = flex.enable_sensing();

        // check pin level is high
        assert!(flex.is_high());
    }

    #[test]
    async fn test_i2c_master(p: Peripherals) {
        use defmt::{error, info};
        use embassy_executor::Spawner;
        use embassy_imxrt::i2c;
        use embassy_time::Timer;
        use embedded_hal_async::i2c::I2c;

        const ACC_ADDR: u8 = 0x1E;

        const ACC_ID_REG: u8 = 0x0D;
        const ACC_CTRL_REG: u8 = 0x2A;
        const ACC_XYZ_DATA_CFG_REG: u8 = 0x0E;
        const ACC_STATUS_REG: u8 = 0x00;

        const ACC_ID: u8 = 0xC7;
        const ACC_STATUS_DATA_READY: u8 = 0xFF;
        {
            let pac = embassy_imxrt::pac::Peripherals::take().unwrap();

            // Ensure SFRO Clock is set to run (power down is cleared)
            pac.sysctl0.pdruncfg0_clr().write(|w| w.sfro_pd().set_bit());

            info!("Enabling GPIO1 clock");
            pac.clkctl1.pscctl1_set().write(|w| w.hsgpio1_clk_set().set_clock());

            // Take GPIO0 out of reset
            info!("Clearing GPIO1 reset");
            pac.rstctl1.prstctl1_clr().write(|w| w.hsgpio1_rst_clr().clr_reset());
        }

        info!("i2c example - Configure GPIOs");
        use embassy_imxrt::gpio::*;

        // Set GPIO1_7 (Reset) as output
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
        info!("Configuring GPIO1_7 as output");
        info!("Configuring GPIO1_7 as low");
        let mut _reset_pin = Output::new(
            p.PIO1_7,
            Level::Low,
            DriveMode::PushPull,
            DriveStrength::Normal,
            SlewRate::Standard,
        );

        // Set GPIO1_5 (Interrupt) as input
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
        info!("Configuring GPIO1_5 as input");
        let _isr_pin = Input::new(p.PIO1_5, Pull::Down, Inverter::Disabled);

        info!("i2c example - I2c::new");
        let mut i2c = i2c::master::I2cMaster::new_async(
            p.FLEXCOMM2,
            p.PIO0_18,
            p.PIO0_17,
            i2c::master::Speed::Standard,
            p.DMA0_CH5,
        )
        .unwrap();

        // Read WHO_AM_I register, 0x0D to get value 0xC7 (1100 0111)
        info!("i2c example - ACC WHO_AM_I register check");
        let mut reg = [0u8; 1];
        reg[0] = 0xAA;
        let result = i2c.write_read(ACC_ADDR, &[ACC_ID_REG], &mut reg).await;
        if result.is_ok() && reg[0] == ACC_ID {
            info!("i2c example - Read WHO_AM_I register: {:02X}", reg[0]);
        } else {
            error!("i2c example - Error reading WHO_AM_I register {}", result.unwrap_err());
            assert!(false);
        }

        //  Write 0x00 to accelerometer control register 1
        info!("i2c example - Write 0x00 to ACC control register");
        let mut reg = [0u8; 2];
        reg[0] = ACC_CTRL_REG;
        reg[1] = 0x00;
        let result = i2c.write(ACC_ADDR, &reg).await;
        if result.is_ok() {
            info!("i2c example - Write ctrl reg");
        } else {
            error!("i2c example - Error writing ctrl reg {}", result.unwrap_err());
            assert!(false);
        }

        //  Write 0x01 to XYZ_DATA_CFG register, set acc range of +/- 4g range and no hpf
        /*  [7]: reserved */
        /*  [6]: reserved */
        /*  [5]: reserved */
        /*  [4]: hpf_out=0 */
        /*  [3]: reserved */
        /*  [2]: reserved */
        /*  [1-0]: fs=01 for accelerometer range of +/-4g range with 0.488mg/LSB */
        /*  databyte = 0x01; */
        info!("i2c example - Write 0x01 to ACC XYZ_DATA_CFG register");
        let mut reg = [0u8; 2];
        reg[0] = ACC_XYZ_DATA_CFG_REG;
        reg[1] = 0x01;
        let result = i2c.write(ACC_ADDR, &reg).await;
        if result.is_ok() {
            info!("i2c example - Write xyz data cfg reg");
        } else {
            error!("i2c example - Error xyz data cfg reg {}", result.unwrap_err());
            assert!(false);
        }

        // Write 0x0D to accelerometer control register
        /*  [7-6]: aslp_rate=00 */
        /*  [5-3]: dr=001 for 200Hz data rate (when in hybrid mode) */
        /*  [2]: lnoise=1 for low noise mode */
        /*  [1]: f_read=0 for normal 16 bit reads */
        /*  [0]: active=1 to take the part out of standby and enable sampling */
        /*   databyte = 0x0D; */
        info!("i2c example - Write 0x0D to ACC control register");
        let mut reg = [0u8; 2];
        reg[0] = ACC_CTRL_REG;
        reg[1] = 0x0D;
        let result = i2c.write(ACC_ADDR, &reg).await;
        if result.is_ok() {
            info!("i2c example - Write ctrl reg");
        } else {
            error!("i2c example - Error writing control reg {}", result.unwrap_err());
            assert!(false);
        }

        info!("i2c example - Read ACC status register until is ready (0xFF)");
        let mut reg = [0u8; 1];
        reg[0] = 0xAA;
        while reg[0] != ACC_STATUS_DATA_READY {
            let result = i2c.write_read(ACC_ADDR, &[ACC_STATUS_REG], &mut reg).await;
            if result.is_ok() {
                info!("i2c example - Read status register: {:02X}", reg[0]);
            } else {
                error!("i2c example - Error reading status register {}", result.unwrap_err());
                assert!(false);
            }
        }

        /* Accelerometer status register, first byte always 0xFF, then X:Y:Z each 2 bytes, in total 7 bytes */
        info!("i2c example - Read XYZ data from ACC status register");
        for _ in 0..10 {
            let mut reg: [u8; 7] = [0xAA; 7];
            let result = i2c.write_read(ACC_ADDR, &[ACC_STATUS_REG], &mut reg).await;
            if result.is_ok() {
                info!("i2c example - Read XYZ data: {:02X}", reg);
            } else {
                error!("i2c example - Error reading XYZ data {}", result.unwrap_err());
                assert!(false);
            }
        }

        info!("i2c example - Done!  Exiting...");
    }
}
