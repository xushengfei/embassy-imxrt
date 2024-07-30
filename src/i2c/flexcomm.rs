//! I2C (Inter-Integrated Circuit) bus Flexcomm Peripheral Setup

use super::instance::{Instance, SealedInstance};

// Cortex-M33 Flexcomm0
impl Instance for crate::peripherals::FLEXCOMM0 {}
impl SealedInstance for crate::peripherals::FLEXCOMM0 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm0::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c0::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 0_2 for SDA
        //
        // Pin is configured as FC0_RXD_SDA_MOSI_DATA
        p.iopctl.pio0_2().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 0_1 for SCL
        //
        // Pin is configured as FC0_TXD_SCL_MISO_WS
        p.iopctl.pio0_1().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(0).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc0_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm0");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm1
impl Instance for crate::peripherals::FLEXCOMM1 {}
impl SealedInstance for crate::peripherals::FLEXCOMM1 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm1::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c1::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 0_9 for SDA
        //
        // Pin is configured as FC1_RXD_SDA_MOSI_DATA
        p.iopctl.pio0_9().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 0_8 for SCL
        //
        // Pin is configured as FC1_TXD_SCL_MISO_WS
        p.iopctl.pio0_8().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(1).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc1_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm1");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm2
impl Instance for crate::peripherals::FLEXCOMM2 {}
impl SealedInstance for crate::peripherals::FLEXCOMM2 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm2::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c2::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 0_17 for SDA
        //
        // Pin is configured as FC2_CTS_SDA_SSEL0
        p.iopctl.pio0_17().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 0_18 for SDA
        //
        // Pin is configured as FC2_RTS_SCL_SSEL1
        p.iopctl.pio0_18().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc2_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        clkctl1.pscctl0_set().write(|w| w.fc2_clk_set().set_bit());
        clkctl1.flexcomm(2).fcfclksel().write(|w| w.sel().sfro_clk());
        clkctl1.flexcomm(2).frgclksel().write(|w| w.sel().sfro_clk());
        clkctl1.flexcomm(2).frgctl().write(|w| unsafe { w.mult().bits(0) });

        let rstctl1 = unsafe { &*crate::pac::Rstctl1::ptr() };
        rstctl1.prstctl0_clr().write(|w| w.flexcomm2_rst_clr().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if pselid.read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm2");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm0
impl Instance for crate::peripherals::FLEXCOMM3 {}
impl SealedInstance for crate::peripherals::FLEXCOMM3 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm3::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c3::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 0_23 for SDA
        //
        // Pin is configured as FC3_RXD_SDA_MOSI_DATA
        p.iopctl.pio0_23().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 0_22 for SCL
        //
        // Pin is configured as FC3_TXD_SCL_MISO_WS
        p.iopctl.pio0_22().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(3).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc3_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm3");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm0
impl Instance for crate::peripherals::FLEXCOMM4 {}
impl SealedInstance for crate::peripherals::FLEXCOMM4 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm4::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c4::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 0_30 for SDA
        //
        // Pin is configured as FC4_RXD_SDA_MOSI_DATA
        p.iopctl.pio0_30().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 0_29 for SCL
        //
        // Pin is configured as FC4_TXD_SCL_MISO_WS
        p.iopctl.pio0_29().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(4).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc4_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm4");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm0
impl Instance for crate::peripherals::FLEXCOMM5 {}
impl SealedInstance for crate::peripherals::FLEXCOMM5 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm5::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c5::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 1_5 for SDA
        //
        // Pin is configured as FC5_RXD_SDA_MOSI_DATA
        p.iopctl.pio1_5().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 1_4 for SCL
        //
        // Pin is configured as FC5_TXD_SCL_MISO_WS
        p.iopctl.pio1_4().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(5).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc5_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm5");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm0
impl Instance for crate::peripherals::FLEXCOMM6 {}
impl SealedInstance for crate::peripherals::FLEXCOMM6 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm6::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c6::RegisterBlock {
        unsafe { &*crate::pac::I2c0::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 3_28 for SDA
        //
        // Pin is configured as FC6_CTS_SDA_SSEL0
        p.iopctl.pio3_28().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 3_29 for SCL
        //
        // Pin is configured as FC6_RTS_SCL_SSEL1
        p.iopctl.pio3_29().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(6).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc6_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm6");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm0
impl Instance for crate::peripherals::FLEXCOMM7 {}
impl SealedInstance for crate::peripherals::FLEXCOMM7 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm7::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c7::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control 4_2 for SDA
        //
        // Pin is configured as FC7_RXD_SDA_MOSI_DATA
        p.iopctl.pio4_2().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control 4_1 for SCL
        //
        // Pin is configured as FC7_TXD_SCL_MISO_WS
        p.iopctl.pio4_1().write(|w| {
            w.fsel()
                .function_1()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(7).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc7_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm7");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}

// Cortex-M33 Flexcomm0
impl Instance for crate::peripherals::FLEXCOMM15 {}
impl SealedInstance for crate::peripherals::FLEXCOMM15 {
    fn flexcomm_regs() -> &'static crate::pac::flexcomm0::RegisterBlock {
        unsafe { &*crate::pac::Flexcomm15::ptr() }
    }

    fn i2c_regs() -> &'static crate::pac::i2c0::RegisterBlock {
        unsafe { &*crate::pac::I2c15::ptr() }
    }

    fn init(p: &crate::pac::Peripherals) {
        // Configure IO Pad Control for SDA
        //
        // Pin is configured as FC15_I2C_SDA
        p.iopctl.fc15_i2c_sda().write(|w| {
            w.fsel()
                .function_0()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // Configure IO Pad Control for SCL
        //
        // Pin is configured as FC15_I2C_SCL
        p.iopctl.fc15_i2c_scl().write(|w| {
            w.fsel()
                .function_0()
                .pupdena()
                .disabled()
                .pupdsel()
                .pull_down()
                .ibena()
                .enabled()
                .slewrate()
                .set_bit()
                .fulldrive()
                .normal_drive()
                .amena()
                .disabled()
                .odena()
                .enabled()
                .iiena()
                .disabled()
        });

        // From Section 21.4 (pg. 544) for Flexcomm in User Manual, enable fc0_clk
        let clkctl1 = unsafe { &*crate::pac::Clkctl1::ptr() };

        let fc2fclksel = clkctl1.flexcomm(15).fcfclksel();
        fc2fclksel.write(|w| w.sel().sfro_clk());

        let pscctl0_set = clkctl1.pscctl0_set();
        pscctl0_set.write(|w| w.fc15_i2c_clk_set().set_bit());

        let pselid = Self::flexcomm_regs().pselid();

        // Check I2C Support
        if Self::flexcomm_regs().pselid().read().i2cpresent().bit_is_clear() {
            panic!("I2C not present in Flexcomm15");
        }

        // Set I2C mode
        pselid.write(|w| w.persel().i2c());
    }
}
