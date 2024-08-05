//! I2C (Inter-Integrated Circuit) bus support

use embassy_hal_internal::{Peripheral, PeripheralRef};

/// I2C Struct
pub struct I2c<'d, T> {
    peripheral: PeripheralRef<'d, T>,
}

impl<'d, T> I2c<'d, T> {
    fn init(&self) -> () {}

    pub fn new(flexcomm: impl Peripheral<P = T> + 'd) -> Self {
        let r = Self {
            peripheral: flexcomm.into_ref(),
        };

        r.init();
        r
    }
}
