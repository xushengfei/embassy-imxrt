//! ADC

use crate::peripherals::ADC0;
use core::marker::PhantomData;
use embassy_hal_internal::Peripheral;

/// ADC driver.
pub struct Adc<'d> {
    _adc0: PhantomData<&'d ADC0>,
}

impl<'d> Adc<'d> {
    fn init() {}
}

impl<'d> Adc<'d> {
    /// Create ADC driver.
    pub fn new(_adc: impl Peripheral<P = ADC0> + 'd) -> Self {
        Self::init();

        Self { _adc0: PhantomData }
    }
}
