#[allow(non_camel_case_types)]
pub enum Frequency {
    F100_kHz,
    F400_kHz,
}

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
