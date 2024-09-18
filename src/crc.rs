//! Cyclic Redundancy Check (CRC)

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::pac::CrcEngine;
use crate::peripherals::CRC;
use crate::Peripheral;

/// CRC driver.
pub struct Crc<'d> {
    _peripheral: PeripheralRef<'d, CRC>,
    _config: Config,
    crc_engine: CrcEngine,
}

/// CRC configuration
pub struct Config {
    polynomial: Polynomial,
    bit_order_input_reverse: bool,
    input_complement: bool,
    bit_order_crc_reverse: bool,
    crc_complement: bool,
    seed: u32,
}

impl Config {
    /// Create a new CRC config.
    pub fn new(
        polynomial: Polynomial,
        bit_order_input_reverse: bool,
        input_complement: bool,
        bit_order_crc_reverse: bool,
        crc_complement: bool,
        seed: u32,
    ) -> Self {
        Config {
            polynomial,
            bit_order_input_reverse,
            input_complement,
            bit_order_crc_reverse,
            crc_complement,
            seed,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polynomial: Polynomial::default(),
            bit_order_input_reverse: false,
            input_complement: false,
            bit_order_crc_reverse: false,
            crc_complement: false,
            seed: 0xffffffff,
        }
    }
}

/// CRC polynomial
#[derive(Debug, Copy, Clone, Default)]
pub enum Polynomial {
    /// CRC-32: 0x04C11DB7
    #[default]
    Crc32,
    /// CRC-16: 0x8005
    Crc16,
    /// CRC-CCITT: 0x1021
    CrcCcitt,
}

impl From<Polynomial> for u8 {
    fn from(polynomial: Polynomial) -> u8 {
        match polynomial {
            Polynomial::Crc16 => 1,
            Polynomial::CrcCcitt => 0,
            _ => 2,
        }
    }
}

impl<'d> Crc<'d> {
    /// Instantiates new CRC peripheral and initializes to default values.
    pub fn new(peripheral: impl Peripheral<P = CRC> + 'd, config: Config) -> Self {
        let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };
        let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };

        clkctl1.pscctl1_set().write(|w| w.crc_clk_set().set_clock());
        rstctl1.prstctl1_clr().write(|w| w.crc_rst_clr().clr_reset());

        into_ref!(peripheral);

        let mut instance = Self {
            _peripheral: peripheral,
            _config: config,
            crc_engine: unsafe { CrcEngine::steal() },
        };

        instance.reconfigure();
        instance
    }

    /// Reconfigured the CRC peripheral.
    fn reconfigure(&mut self) {
        self.crc_engine.mode().write(|w| {
            if self._config.bit_order_input_reverse {
                w.bit_rvs_wr().set_bit();
            } else {
                w.bit_rvs_wr().clear_bit();
            }

            if self._config.input_complement {
                w.cmpl_wr().set_bit();
            } else {
                w.cmpl_wr().clear_bit();
            }

            if self._config.bit_order_crc_reverse {
                w.bit_rvs_sum().set_bit();
            } else {
                w.bit_rvs_sum().clear_bit();
            }

            if self._config.crc_complement {
                w.bit_rvs_sum().set_bit();
            } else {
                w.bit_rvs_sum().clear_bit();
            }

            unsafe { w.crc_poly().bits(self._config.polynomial.into()) };

            w
        });

        // Init CRC value
        self.crc_engine
            .seed()
            .write(|w| unsafe { w.crc_seed().bits(self._config.seed) });
    }

    /// Feeds a byte into the CRC peripheral. Returns the computed checksum.
    pub fn feed_byte(&mut self, byte: u8) -> u32 {
        self.crc_engine
            .wr_data()
            .write(|w| unsafe { w.crc_wr_data().bits(u32::from(byte)) });

        self.crc_engine.sum().read().bits()
    }

    /// Feeds an slice of bytes into the CRC peripheral. Returns the computed checksum.
    pub fn feed_bytes(&mut self, bytes: &[u8]) -> u32 {
        for byte in bytes {
            self.crc_engine
                .wr_data()
                .write(|w| unsafe { w.crc_wr_data().bits(u32::from(*byte)) });
        }

        self.crc_engine.sum().read().bits()
    }

    /// Feeds a halfword into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfword(&mut self, halfword: u16) -> u32 {
        self.crc_engine
            .wr_data()
            .write(|w| unsafe { w.crc_wr_data().bits(u32::from(halfword)) });

        self.crc_engine.sum().read().bits()
    }

    /// Feeds an slice of halfwords into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfwords(&mut self, halfwords: &[u16]) -> u32 {
        for halfword in halfwords {
            self.crc_engine
                .wr_data()
                .write(|w| unsafe { w.crc_wr_data().bits(u32::from(*halfword)) });
        }

        self.crc_engine.sum().read().bits()
    }

    /// Feeds a words into the CRC peripheral. Returns the computed checksum.
    pub fn feed_word(&mut self, word: u32) -> u32 {
        self.crc_engine
            .wr_data()
            .write(|w| unsafe { w.crc_wr_data().bits(word) });

        self.crc_engine.sum().read().bits()
    }

    /// Feeds an slice of words into the CRC peripheral. Returns the computed checksum.
    pub fn feed_words(&mut self, words: &[u32]) -> u32 {
        for word in words {
            self.crc_engine
                .wr_data()
                .write(|w| unsafe { w.crc_wr_data().bits(*word) });
        }

        self.crc_engine.sum().read().bits()
    }
}
