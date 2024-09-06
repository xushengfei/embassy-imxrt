//! Implements I2C Master and Slave Functionalities

pub mod slave;
pub use slave::*;

use crate::{pac, Peripheral, PeripheralRef};

#[allow(private_bounds)]
trait I2CPeripheral: Peripheral {
    fn i2c() -> &'static pac::i2c0::RegisterBlock;

    // todo replace with flexcomm interface
    fn fc() -> &'static pac::flexcomm0::RegisterBlock;
    fn enable_flexcomm();
}

/// Bus speed (nominal SCL, no clock stretching)
pub enum Speed {
    /// 100 kbit/s
    Standard,

    /// 400 kbit/s
    Fast,

    /// 1 Mbit/s
    FastPlus,

    /// 3.4Mbit/s only available for slave devices
    High,
}

/// I2C address type
#[derive(Copy, Clone, Debug)]
pub struct Address(u8);

impl Address {
    /// Construct an address type
    pub const fn new(addr: u8) -> Option<Self> {
        match addr {
            0x08..=0x77 => Some(Self(addr)),
            _ => None,
        }
    }

    /// interpret address as a read command
    pub fn read(&self) -> u8 {
        (self.0 << 1) | 1
    }

    /// interpret address as a write command
    pub fn write(&self) -> u8 {
        self.0 << 1
    }
}

impl From<Address> for u8 {
    fn from(value: Address) -> Self {
        value.0
    }
}

macro_rules! impl_i2c {
    ($flexcomm_upper:ident, $flexcomm_lower:ident, $flexcomm_rst:ident, $i2c:ident, $fclock:ident, $fc:expr) => {
        impl I2CPeripheral for crate::peripherals::$flexcomm_upper {
            fn i2c() -> &'static pac::i2c0::RegisterBlock {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$i2c::ptr() }
            }

            // todo replace with flexcomm interface
            fn fc() -> &'static pac::flexcomm0::RegisterBlock {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                unsafe { &*crate::pac::$flexcomm_lower::ptr() }
            }

            // todo replace with flexcomm interface
            fn enable_flexcomm() {
                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };
                clkctl1.pscctl0_set().write(|w| w.$fclock().set_clock());
                clkctl1.flexcomm($fc).fcfclksel().write(|w| w.sel().sfro_clk());

                // SAFETY: safe from single executor, enforce via peripheral reference lifetime tracking
                let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };
                rstctl1.prstctl0_clr().write(|w| w.$flexcomm_rst().set_bit());

                Self::fc().pselid().write(|w| w.persel().i2c());
            }
        }
    };
}

impl_i2c!(FLEXCOMM0, Flexcomm0, flexcomm0_rst_clr, I2c0, fc0_clk_set, 0);
impl_i2c!(FLEXCOMM1, Flexcomm1, flexcomm1_rst_clr, I2c1, fc1_clk_set, 1);
impl_i2c!(FLEXCOMM2, Flexcomm2, flexcomm2_rst_clr, I2c2, fc2_clk_set, 2);
impl_i2c!(FLEXCOMM3, Flexcomm3, flexcomm3_rst_clr, I2c3, fc3_clk_set, 3);
impl_i2c!(FLEXCOMM4, Flexcomm4, flexcomm4_rst_clr, I2c4, fc4_clk_set, 4);
impl_i2c!(FLEXCOMM5, Flexcomm5, flexcomm5_rst_clr, I2c5, fc5_clk_set, 5);
impl_i2c!(FLEXCOMM6, Flexcomm6, flexcomm6_rst_clr, I2c6, fc6_clk_set, 6);
impl_i2c!(FLEXCOMM7, Flexcomm7, flexcomm7_rst_clr, I2c7, fc7_clk_set, 7);
