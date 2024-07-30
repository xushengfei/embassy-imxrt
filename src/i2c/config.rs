//! I2C (Inter-Integrated Circuit) bus Configuration object

/// Frequency for I2C Communications
#[allow(non_camel_case_types)]
pub enum Frequency {
    /// 100 kHz operation
    F100_kHz,

    /// 400 kHz operation
    F400_kHz,
}

/// Configuration struct to set I2c bus master options
pub struct Config {
    /// Frequency for I2C Communications
    pub frequency: Frequency,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::F100_kHz,
        }
    }
}
