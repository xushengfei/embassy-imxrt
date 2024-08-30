//! GPIO driver.
#![macro_use]
use core::convert::Infallible;

use crate::{pac, peripherals, Peripheral};

use crate::iopctl::{*};


/// Struct to configure [IOPin] by selecting its fields - [Pull], [SlewRate], [DriveStrength], [DriveMode], [Polarity], and buffer
/// 
/// default() sets configuration fields as -
///  [Pull::None], [SlewRate::Standard], [DriveStrength::Normal], [DriveMode::PushPull], [Polarity::ActiveHigh] and Buffer enabled
pub struct Config{
    /// Set the [Pull] configuration for pin
    pub pull : Pull,

    /// Set the [SlewRate] configuration for pin
    pub slew_rate : SlewRate,

    /// Set the [DriveStrength] configuration for pin
    pub drive_strength : DriveStrength,

    /// Set the [DriveMode] configuration for pin
    pub drive_mode : DriveMode,

    /// Set the [Polarity] configuration for pin
    pub polarity : Polarity,

    /// Set the buffer as enabled or disabled configuration for pin
    pub buffer : bool,

}

impl Default for Config {
    fn default() -> Self {
        Self {
            pull : Pull::None,
            slew_rate : SlewRate::Standard,
            drive_strength : DriveStrength::Normal,
            drive_mode : DriveMode::PushPull,
            polarity : Polarity::ActiveHigh,
            buffer : true,
        }
    }
}

/// A GPIO port with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Port {
    /// Port 0, available on all packages
    Port0,

    /// Port 1, available on all packages
    Port1,

    /// Port 2, available on all packages
    Port2,

    /// Port 3, available on 114-pin and 249-pin packages
    Port3,

    /// Port 4, available on 249-pin packages
    Port4,

    /// Port 7, available on 249-pin packages
    Port7,
}

/// Digital input or output level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Logical low.
    Low,
    /// Logical high.
    High,
}

impl From<bool> for Level {
    fn from(val: bool) -> Self {
        match val {
            true => Self::High,
            false => Self::Low,
        }
    }
}

impl From<Level> for bool {
    fn from(level: Level) -> bool {
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}

/// GPIO input driver.
pub struct Input {
    pin: IOPin,
}


impl Input {
    /// Create GPIO Input driver for a [Pin] with the provided [Level] configuration.
    pub fn new(pin: impl Pin, initial_config: Config) -> Self {
        pin.set_config(initial_config);
        let pin : IOPin = pin.into();
        pin.set_as_input();
        Self { pin }
    }

    /// Get whether the pin input [Level] is High].
    #[inline]
    pub fn is_input_high(&self) -> bool {
        !self.is_input_low()
    }

    /// Get whether the pin input [Level] is set to [Level::Low].
    #[inline]
    pub fn is_input_low(&self) -> bool {
        let port = self.pin.port_index();
        let pin = self.pin.pin_index();
        self.pin.block().b(port).b_(pin).read() == 0
    }

    /// Get the pin input [Level].
    #[inline]
    pub fn get_input_level(&self) -> Level {
        self.is_input_high().into()
    }

    /// Put the input pin into disconnected mode.
    pub fn set_as_disconnected(&mut self) {
        self.pin.disconnect();
    }
}

/// GPIO output drivber.
pub struct Output {
    pin: IOPin,
}

impl Output {
    /// Create GPIO output driver for a [Pin] with the provided [Level] configuration.
    /// 
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    pub fn new(pin: impl Pin, initial_config: Config) -> Self {
        pin.set_config(initial_config);
        let pin : IOPin = pin.into();
        pin.set_as_output();
        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_output_high(&mut self) {
        let port = self.pin.port_index();
        let pin = self.pin.pin_index();
        self.pin
            .block()
            .set(port)
            .modify(|r, w| unsafe { w.setp().bits(r.setp().bits() | (1 << pin)) });
    }

    /// Set the output as low.
    #[inline]
    pub fn set_output_low(&mut self) {
        let port = self.pin.port_index();
        let pin = self.pin.pin_index();
        self.pin
            .block()
            .clr(port)
            .write(|w| unsafe { w.clrp().bits(1 << pin) });
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        let port = self.pin.port_index();
        let pin = self.pin.pin_index();
        self.pin.block().not(port).write(|w| unsafe { w.notp().bits(1 << pin) });
    }

    /// Set the output [Level] as [Level::Low] or [Level::High]
    #[inline]
    pub fn set_output_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_output_low(),
            Level::High => self.set_output_high(),
        }
    }

    /// Get whether the output [Level] is set to [Level::High].
    #[inline]
    pub fn is_output_high(&self) -> bool {
        !self.is_output_low()
    }

    /// Get whether the output [Level] is set to [Level::Low].
    #[inline]
    pub fn is_output_low(&self) -> bool {
        let port = self.pin.port_index();
        let pin = self.pin.pin_index();
        self.pin.block().b(port).b_(pin).read() == 0
    }

    /// Get the current output [Level].
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_output_high().into()
    }

    /// Put the output pin into disconnected mode.
    pub fn set_as_disconnected(&mut self) {
        self.pin.disconnect();
    }
}

trait SealedPin {

    #[inline]
    fn block(&self) -> &pac::gpio::RegisterBlock {
        unsafe { &*pac::Gpio::ptr() }
    }

}
impl SealedPin for IOPin{}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an [IOPin].
#[allow(private_bounds)]
pub trait Pin: Peripheral<P = Self> + Into<IOPin> + Sized + 'static {
    /// To set initial configuration of the peripheral Pin before using as [IOPin] 
    /// using [Config] struct fields
    fn set_config(&self, initial_config : Config);
}

/// GPIO Pins
enum IOPin{
    PIO0_0,PIO0_1,PIO0_2,PIO0_3,PIO0_4,PIO0_5,PIO0_6,PIO0_7,PIO0_8,PIO0_9,PIO0_10,PIO0_11,PIO0_12,PIO0_13,PIO0_14,PIO0_15,PIO0_16,PIO0_17,PIO0_18,PIO0_19,PIO0_20,PIO0_21,PIO0_22,PIO0_23,PIO0_24,PIO0_25,PIO0_26,PIO0_27,PIO0_28,PIO0_29,PIO0_30,PIO0_31,
    PIO1_0,PIO1_1,PIO1_2,PIO1_3,PIO1_4,PIO1_5,PIO1_6,PIO1_7,PIO1_8,PIO1_9,PIO1_10,PIO1_11,PIO1_12,PIO1_13,PIO1_14,PIO1_15,PIO1_16,PIO1_17,PIO1_18,PIO1_19,PIO1_20,PIO1_21,PIO1_22,PIO1_23,PIO1_24,PIO1_25,PIO1_26,PIO1_27,PIO1_28,PIO1_29,PIO1_30,PIO1_31,
    PIO2_0,PIO2_1,PIO2_2,PIO2_3,PIO2_4,PIO2_5,PIO2_6,PIO2_7,PIO2_8,PIO2_9,PIO2_10,PIO2_11,PIO2_12,PIO2_13,PIO2_14,PIO2_15,PIO2_16,PIO2_17,PIO2_18,PIO2_19,PIO2_20,PIO2_21,PIO2_22,PIO2_23,PIO2_24,PIO2_25,PIO2_26,PIO2_27,PIO2_28,PIO2_29,PIO2_30,PIO2_31,
    PIO3_0,PIO3_1,PIO3_2,PIO3_3,PIO3_4,PIO3_5,PIO3_6,PIO3_7,PIO3_8,PIO3_9,PIO3_10,PIO3_11,PIO3_12,PIO3_13,PIO3_14,PIO3_15,PIO3_16,PIO3_17,PIO3_18,PIO3_19,PIO3_20,PIO3_21,PIO3_22,PIO3_23,PIO3_24,PIO3_25,PIO3_26,PIO3_27,PIO3_28,PIO3_29,PIO3_30,PIO3_31,
    PIO4_0,PIO4_1,PIO4_2,PIO4_3,PIO4_4,PIO4_5,PIO4_6,PIO4_7,PIO4_8,PIO4_9,PIO4_10,
    PIO7_24,PIO7_25,PIO7_26,PIO7_27,PIO7_28,PIO7_29,PIO7_30,PIO7_31,
}

impl IOPin{
    /// Get the [Port] of the [IOPin]
    #[inline]
    fn port(&self) -> Port {
        match self{
            Self::PIO0_0|Self::PIO0_1|Self::PIO0_2|Self::PIO0_3|Self::PIO0_4|Self::PIO0_5|Self::PIO0_6|Self::PIO0_7|Self::PIO0_8|Self::PIO0_9|Self::PIO0_10|Self::PIO0_11|Self::PIO0_12|Self::PIO0_13|Self::PIO0_14|Self::PIO0_15|Self::PIO0_16|Self::PIO0_17|Self::PIO0_18|Self::PIO0_19|Self::PIO0_20|Self::PIO0_21|Self::PIO0_22|Self::PIO0_23|Self::PIO0_24|Self::PIO0_25|Self::PIO0_26|Self::PIO0_27|Self::PIO0_28|Self::PIO0_29|Self::PIO0_30|Self::PIO0_31 => Port::Port0,
            Self::PIO1_0|Self::PIO1_1|Self::PIO1_2|Self::PIO1_3|Self::PIO1_4|Self::PIO1_5|Self::PIO1_6|Self::PIO1_7|Self::PIO1_8|Self::PIO1_9|Self::PIO1_10|Self::PIO1_11|Self::PIO1_12|Self::PIO1_13|Self::PIO1_14|Self::PIO1_15|Self::PIO1_16|Self::PIO1_17|Self::PIO1_18|Self::PIO1_19|Self::PIO1_20|Self::PIO1_21|Self::PIO1_22|Self::PIO1_23|Self::PIO1_24|Self::PIO1_25|Self::PIO1_26|Self::PIO1_27|Self::PIO1_28|Self::PIO1_29|Self::PIO1_30|Self::PIO1_31 => Port::Port1,
            Self::PIO2_0|Self::PIO2_1|Self::PIO2_2|Self::PIO2_3|Self::PIO2_4|Self::PIO2_5|Self::PIO2_6|Self::PIO2_7|Self::PIO2_8|Self::PIO2_9|Self::PIO2_10|Self::PIO2_11|Self::PIO2_12|Self::PIO2_13|Self::PIO2_14|Self::PIO2_15|Self::PIO2_16|Self::PIO2_17|Self::PIO2_18|Self::PIO2_19|Self::PIO2_20|Self::PIO2_21|Self::PIO2_22|Self::PIO2_23|Self::PIO2_24|Self::PIO2_25|Self::PIO2_26|Self::PIO2_27|Self::PIO2_28|Self::PIO2_29|Self::PIO2_30|Self::PIO2_31 => Port::Port2,
            Self::PIO3_0|Self::PIO3_1|Self::PIO3_2|Self::PIO3_3|Self::PIO3_4|Self::PIO3_5|Self::PIO3_6|Self::PIO3_7|Self::PIO3_8|Self::PIO3_9|Self::PIO3_10|Self::PIO3_11|Self::PIO3_12|Self::PIO3_13|Self::PIO3_14|Self::PIO3_15|Self::PIO3_16|Self::PIO3_17|Self::PIO3_18|Self::PIO3_19|Self::PIO3_20|Self::PIO3_21|Self::PIO3_22|Self::PIO3_23|Self::PIO3_24|Self::PIO3_25|Self::PIO3_26|Self::PIO3_27|Self::PIO3_28|Self::PIO3_29|Self::PIO3_30|Self::PIO3_31 => Port::Port3,
            Self::PIO4_0|Self::PIO4_1|Self::PIO4_2|Self::PIO4_3|Self::PIO4_4|Self::PIO4_5|Self::PIO4_6|Self::PIO4_7|Self::PIO4_8|Self::PIO4_9|Self::PIO4_10 => Port::Port4,
            Self::PIO7_24|Self::PIO7_25|Self::PIO7_26|Self::PIO7_27|Self::PIO7_28|Self::PIO7_29|Self::PIO7_30|Self::PIO7_31 => Port::Port7,
        }
    }

    /// Get the port number of [IOPin]
    #[inline]
    fn port_index(&self) -> usize{
        match self.port(){
            Port::Port0 => 0,
            Port::Port1 => 1,
            Port::Port2 => 2,
            Port::Port3 => 3,
            Port::Port4 => 4,
            Port::Port7 => 7,
        }
    }
    
    /// Get the pin number (0..31) of [IOPin]
    #[inline]
    fn pin_index(&self) -> usize {
        match self {
            Self::PIO0_0|Self::PIO1_0|Self::PIO2_0|Self::PIO3_0|Self::PIO4_0 => 0,
            Self::PIO0_1|Self::PIO1_1|Self::PIO2_1|Self::PIO3_1|Self::PIO4_1 => 1,
            Self::PIO0_2|Self::PIO1_2|Self::PIO2_2|Self::PIO3_2|Self::PIO4_2 => 2,
            Self::PIO0_3|Self::PIO1_3|Self::PIO2_3|Self::PIO3_3|Self::PIO4_3 => 3,
            Self::PIO0_4|Self::PIO1_4|Self::PIO2_4|Self::PIO3_4|Self::PIO4_4 => 4,
            Self::PIO0_5|Self::PIO1_5|Self::PIO2_5|Self::PIO3_5|Self::PIO4_5 => 5,
            Self::PIO0_6|Self::PIO1_6|Self::PIO2_6|Self::PIO3_6|Self::PIO4_6 => 6,
            Self::PIO0_7|Self::PIO1_7|Self::PIO2_7|Self::PIO3_7|Self::PIO4_7 => 7,
            Self::PIO0_8|Self::PIO1_8|Self::PIO2_8|Self::PIO3_8|Self::PIO4_8 => 8,
            Self::PIO0_9|Self::PIO1_9|Self::PIO2_9|Self::PIO3_9|Self::PIO4_9 => 9,
            Self::PIO0_10|Self::PIO1_10|Self::PIO2_10|Self::PIO3_10|Self::PIO4_10 => 10,
            Self::PIO0_11|Self::PIO1_11|Self::PIO2_11|Self::PIO3_11 => 11,
            Self::PIO0_12|Self::PIO1_12|Self::PIO2_12|Self::PIO3_12 => 12,
            Self::PIO0_13|Self::PIO1_13|Self::PIO2_13|Self::PIO3_13 => 13,
            Self::PIO0_14|Self::PIO1_14|Self::PIO2_14|Self::PIO3_14 => 14,
            Self::PIO0_15|Self::PIO1_15|Self::PIO2_15|Self::PIO3_15 => 15,
            Self::PIO0_16|Self::PIO1_16|Self::PIO2_16|Self::PIO3_16 => 16,
            Self::PIO0_17|Self::PIO1_17|Self::PIO2_17|Self::PIO3_17 => 17,
            Self::PIO0_18|Self::PIO1_18|Self::PIO2_18|Self::PIO3_18 => 18,
            Self::PIO0_19|Self::PIO1_19|Self::PIO2_19|Self::PIO3_19 => 19,
            Self::PIO0_20|Self::PIO1_20|Self::PIO2_20|Self::PIO3_20 => 20,
            Self::PIO0_21|Self::PIO1_21|Self::PIO2_21|Self::PIO3_21 => 21,
            Self::PIO0_22|Self::PIO1_22|Self::PIO2_22|Self::PIO3_22 => 22,
            Self::PIO0_23|Self::PIO1_23|Self::PIO2_23|Self::PIO3_23 => 23,
            Self::PIO0_24|Self::PIO1_24|Self::PIO2_24|Self::PIO3_24|Self::PIO7_24 => 24,
            Self::PIO0_25|Self::PIO1_25|Self::PIO2_25|Self::PIO3_25|Self::PIO7_25 => 25,
            Self::PIO0_26|Self::PIO1_26|Self::PIO2_26|Self::PIO3_26|Self::PIO7_26 => 26,
            Self::PIO0_27|Self::PIO1_27|Self::PIO2_27|Self::PIO3_27|Self::PIO7_27 => 27,
            Self::PIO0_28|Self::PIO1_28|Self::PIO2_28|Self::PIO3_28|Self::PIO7_28 => 28,
            Self::PIO0_29|Self::PIO1_29|Self::PIO2_29|Self::PIO3_29|Self::PIO7_29 => 29,
            Self::PIO0_30|Self::PIO1_30|Self::PIO2_30|Self::PIO3_30|Self::PIO7_30 => 30,
            Self::PIO0_31|Self::PIO1_31|Self::PIO2_31|Self::PIO3_31|Self::PIO7_31 => 31,
        }
    }

    /// Configure Pin as [Input]
    fn set_as_input(&self) {
        let port = self.port_index();
        let pin = self.pin_index();
        self.block()
            .dir(port)
            .modify(|r, w| 
                // SAFETY: No other driver should modify or write to the same [Port] register simultaneously
                unsafe { w.dirp().bits(r.dirp().bits() & !(1 << pin)) });
    }

    /// Configure Pin as [Output]
    fn set_as_output(&self) {
        let port = self.port_index();
        let pin = self.pin_index();
        self.block()
            .dir(port)
            .modify(|r, w| 
                // SAFETY: No other driver should modify or write to the same [Port] register simultaneously
                unsafe { w.dirp().bits(r.dirp().bits() | (1 << pin)) });
    }

    /// Disconnect the pin
    fn disconnect(&mut self){
        // bring pin back to reset state
        // SAFETY: 
        // Only one GPIO driver calls this function at a given time
        unsafe { RawPin::new(self.port_index() as u8, self.pin_index() as u8).reset() };
    }
}

impl Drop for IOPin {
    fn drop(&mut self) {
        self.disconnect();
        
    }
}

impl Port {
    /// Enables the GPIO [Port] 0..7
    pub fn init(port: Port) {
        // Enable GPIO clocks and take them out of reset
        let r = unsafe { &*(pac::Clkctl1::ptr()) };
        let t = unsafe { &*(pac::Rstctl1::ptr()) };
        match port {
            Port::Port0 => {
                r.pscctl1_set().write(|w| w.hsgpio0_clk_set().set_clock());
                t.prstctl1_clr().write(|w| w.hsgpio0_rst_clr().clr_reset());
            }
            Port::Port1 => {
                r.pscctl1_set().write(|w| w.hsgpio1_clk_set().set_clock());
                t.prstctl1_clr().write(|w| w.hsgpio1_rst_clr().clr_reset());
            }
            Port::Port2 => {
                r.pscctl1_set().write(|w| w.hsgpio2_clk_set().set_clock());
                t.prstctl1_clr().write(|w| w.hsgpio2_rst_clr().clr_reset());
            }
            Port::Port3 => {
                r.pscctl1_set().write(|w| w.hsgpio3_clk_set().set_clock());
                t.prstctl1_clr().write(|w| w.hsgpio3_rst_clr().clr_reset());
            }
            Port::Port4 => {
                r.pscctl1_set().write(|w| w.hsgpio4_clk_set().set_clock());
                t.prstctl1_clr().write(|w| w.hsgpio4_rst_clr().clr_reset());
            }
            Port::Port7 => {
                r.pscctl1_set().write(|w| w.hsgpio7_clk_set().set_clock());
                t.prstctl1_clr().write(|w| w.hsgpio7_rst_clr().clr_reset());
            }
        }
    }
}


// ====================

impl embedded_hal_02::digital::v2::InputPin for Input {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_input_high())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_input_low())
    }
}

impl embedded_hal_02::digital::v2::OutputPin for Output {
    type Error = Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_output_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_output_low();
        Ok(())
    }
}

impl embedded_hal_02::digital::v2::StatefulOutputPin for Output {
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_output_high())
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_output_low())
    }
}

impl embedded_hal_02::digital::v2::ToggleableOutputPin for Output {
    type Error = Infallible;
    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl embedded_hal_1::digital::ErrorType for Input {
    type Error = Infallible;
}

impl embedded_hal_1::digital::InputPin for Input {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_input_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_input_low())
    }
}

impl embedded_hal_1::digital::ErrorType for Output {
    type Error = Infallible;
}

impl embedded_hal_1::digital::OutputPin for Output {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_output_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_output_low();
        Ok(())
    }
}

impl embedded_hal_1::digital::StatefulOutputPin for Output {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_output_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_output_low())
    }
}

// ====================

macro_rules! impl_pin {
    ($peripheral:ident, $method:ident, $port_num:expr, $pin_num:expr) => {
        impl crate::gpio::Pin for peripherals::$peripheral {
            fn set_config(&self, config : Config){
                self.set_function(Function::F0)
                    .set_pull(config.pull)
                    .set_drive_strength(config.drive_strength)
                    .set_drive_mode(config.drive_mode)
                    .set_input_polarity(config.polarity)
                    .set_slew_rate(config.slew_rate);
                match config.buffer {
                    true => self.enable_input_buffer(),
                    false => self.disable_input_buffer(),
                };
            }
        }

        impl From<peripherals::$peripheral> for crate::gpio::IOPin {
            fn from(_val: peripherals::$peripheral) -> Self {
                crate::gpio::IOPin::$peripheral
            }
        }
    }
}


// GPIO port 0
impl_pin!(PIO0_0, pio0_0, 0, 0);
impl_pin!(PIO0_1, pio0_1, 0, 1);
impl_pin!(PIO0_2, pio0_2, 0, 2);
impl_pin!(PIO0_3, pio0_3, 0, 3);
impl_pin!(PIO0_4, pio0_4, 0, 4);
impl_pin!(PIO0_5, pio0_5, 0, 5);
impl_pin!(PIO0_6, pio0_6, 0, 6);
impl_pin!(PIO0_7, pio0_7, 0, 7);
impl_pin!(PIO0_8, pio0_8, 0, 8);
impl_pin!(PIO0_9, pio0_9, 0, 9);
impl_pin!(PIO0_10, pio0_10, 0, 10);
impl_pin!(PIO0_11, pio0_11, 0, 11);
impl_pin!(PIO0_12, pio0_12, 0, 12);
impl_pin!(PIO0_13, pio0_13, 0, 13);
impl_pin!(PIO0_14, pio0_14, 0, 14);
impl_pin!(PIO0_15, pio0_15, 0, 15);
impl_pin!(PIO0_16, pio0_16, 0, 16);
impl_pin!(PIO0_17, pio0_17, 0, 17);
impl_pin!(PIO0_18, pio0_18, 0, 18);
impl_pin!(PIO0_19, pio0_19, 0, 19);
impl_pin!(PIO0_20, pio0_20, 0, 20);
impl_pin!(PIO0_21, pio0_21, 0, 21);
impl_pin!(PIO0_22, pio0_22, 0, 22);
impl_pin!(PIO0_23, pio0_23, 0, 23);
impl_pin!(PIO0_24, pio0_24, 0, 24);
impl_pin!(PIO0_25, pio0_25, 0, 25);
impl_pin!(PIO0_26, pio0_26, 0, 26);
impl_pin!(PIO0_27, pio0_27, 0, 27);
impl_pin!(PIO0_28, pio0_28, 0, 28);
impl_pin!(PIO0_29, pio0_29, 0, 29);
impl_pin!(PIO0_30, pio0_30, 0, 30);
impl_pin!(PIO0_31, pio0_31, 0, 31);

// GPIO port 1
impl_pin!(PIO1_0, pio1_0, 1, 0);
impl_pin!(PIO1_1, pio1_1, 1, 1);
impl_pin!(PIO1_2, pio1_2, 1, 2);
impl_pin!(PIO1_3, pio1_3, 1, 3);
impl_pin!(PIO1_4, pio1_4, 1, 4);
impl_pin!(PIO1_5, pio1_5, 1, 5);
impl_pin!(PIO1_6, pio1_6, 1, 6);
impl_pin!(PIO1_7, pio1_7, 1, 7);
impl_pin!(PIO1_8, pio1_8, 1, 8);
impl_pin!(PIO1_9, pio1_9, 1, 9);
impl_pin!(PIO1_10, pio1_10, 1, 10);
impl_pin!(PIO1_11, pio1_11, 1, 11);
impl_pin!(PIO1_12, pio1_12, 1, 12);
impl_pin!(PIO1_13, pio1_13, 1, 13);
impl_pin!(PIO1_14, pio1_14, 1, 14);
impl_pin!(PIO1_15, pio1_15, 1, 15);
impl_pin!(PIO1_16, pio1_16, 1, 16);
impl_pin!(PIO1_17, pio1_17, 1, 17);
impl_pin!(PIO1_18, pio1_18, 1, 18);
impl_pin!(PIO1_19, pio1_19, 1, 19);
impl_pin!(PIO1_20, pio1_20, 1, 20);
impl_pin!(PIO1_21, pio1_21, 1, 21);
impl_pin!(PIO1_22, pio1_22, 1, 22);
impl_pin!(PIO1_23, pio1_23, 1, 23);
impl_pin!(PIO1_24, pio1_24, 1, 24);
impl_pin!(PIO1_25, pio1_25, 1, 25);
impl_pin!(PIO1_26, pio1_26, 1, 26);
impl_pin!(PIO1_27, pio1_27, 1, 27);
impl_pin!(PIO1_28, pio1_28, 1, 28);
impl_pin!(PIO1_29, pio1_29, 1, 29);
impl_pin!(PIO1_30, pio1_30, 1, 30);
impl_pin!(PIO1_31, pio1_31, 1, 31);

// GPIO port 2
impl_pin!(PIO2_0, pio2_0, 2, 0);
impl_pin!(PIO2_1, pio2_1, 2, 1);
impl_pin!(PIO2_2, pio2_2, 2, 2);
impl_pin!(PIO2_3, pio2_3, 2, 3);
impl_pin!(PIO2_4, pio2_4, 2, 4);
impl_pin!(PIO2_5, pio2_5, 2, 5);
impl_pin!(PIO2_6, pio2_6, 2, 6);
impl_pin!(PIO2_7, pio2_7, 2, 7);
impl_pin!(PIO2_8, pio2_8, 2, 8);
impl_pin!(PIO2_9, pio2_9, 2, 9);
impl_pin!(PIO2_10, pio2_10, 2, 10);
impl_pin!(PIO2_11, pio2_11, 2, 11);
impl_pin!(PIO2_12, pio2_12, 2, 12);
impl_pin!(PIO2_13, pio2_13, 2, 13);
impl_pin!(PIO2_14, pio2_14, 2, 14);
impl_pin!(PIO2_15, pio2_15, 2, 15);
impl_pin!(PIO2_16, pio2_16, 2, 16);
impl_pin!(PIO2_17, pio2_17, 2, 17);
impl_pin!(PIO2_18, pio2_18, 2, 18);
impl_pin!(PIO2_19, pio2_19, 2, 19);
impl_pin!(PIO2_20, pio2_20, 2, 20);
impl_pin!(PIO2_21, pio2_21, 2, 21);
impl_pin!(PIO2_22, pio2_22, 2, 22);
impl_pin!(PIO2_23, pio2_23, 2, 23);
impl_pin!(PIO2_24, pio2_24, 2, 24);
impl_pin!(PIO2_25, pio2_25, 2, 25);
impl_pin!(PIO2_26, pio2_26, 2, 26);
impl_pin!(PIO2_27, pio2_27, 2, 27);
impl_pin!(PIO2_28, pio2_28, 2, 28);
impl_pin!(PIO2_29, pio2_29, 2, 29);
impl_pin!(PIO2_30, pio2_30, 2, 30);
impl_pin!(PIO2_31, pio2_31, 2, 31);

// GPIO port 3
impl_pin!(PIO3_0, pio3_0, 3, 0);
impl_pin!(PIO3_1, pio3_1, 3, 1);
impl_pin!(PIO3_2, pio3_2, 3, 2);
impl_pin!(PIO3_3, pio3_3, 3, 3);
impl_pin!(PIO3_4, pio3_4, 3, 4);
impl_pin!(PIO3_5, pio3_5, 3, 5);
impl_pin!(PIO3_6, pio3_6, 3, 6);
impl_pin!(PIO3_7, pio3_7, 3, 7);
impl_pin!(PIO3_8, pio3_8, 3, 8);
impl_pin!(PIO3_9, pio3_9, 3, 9);
impl_pin!(PIO3_10, pio3_10, 3, 10);
impl_pin!(PIO3_11, pio3_11, 3, 11);
impl_pin!(PIO3_12, pio3_12, 3, 12);
impl_pin!(PIO3_13, pio3_13, 3, 13);
impl_pin!(PIO3_14, pio3_14, 3, 14);
impl_pin!(PIO3_15, pio3_15, 3, 15);
impl_pin!(PIO3_16, pio3_16, 3, 16);
impl_pin!(PIO3_17, pio3_17, 3, 17);
impl_pin!(PIO3_18, pio3_18, 3, 18);
impl_pin!(PIO3_19, pio3_19, 3, 19);
impl_pin!(PIO3_20, pio3_20, 3, 20);
impl_pin!(PIO3_21, pio3_21, 3, 21);
impl_pin!(PIO3_22, pio3_22, 3, 22);
impl_pin!(PIO3_23, pio3_23, 3, 23);
impl_pin!(PIO3_24, pio3_24, 3, 24);
impl_pin!(PIO3_25, pio3_25, 3, 25);
impl_pin!(PIO3_26, pio3_26, 3, 26);
impl_pin!(PIO3_27, pio3_27, 3, 27);
impl_pin!(PIO3_28, pio3_28, 3, 28);
impl_pin!(PIO3_29, pio3_29, 3, 29);
impl_pin!(PIO3_30, pio3_30, 3, 30);
impl_pin!(PIO3_31, pio3_31, 3, 31);

// GPIO port 4
impl_pin!(PIO4_0, pio4_0, 4, 0);
impl_pin!(PIO4_1, pio4_1, 4, 1);
impl_pin!(PIO4_2, pio4_2, 4, 2);
impl_pin!(PIO4_3, pio4_3, 4, 3);
impl_pin!(PIO4_4, pio4_4, 4, 4);
impl_pin!(PIO4_5, pio4_5, 4, 5);
impl_pin!(PIO4_6, pio4_6, 4, 6);
impl_pin!(PIO4_7, pio4_7, 4, 7);
impl_pin!(PIO4_8, pio4_8, 4, 8);
impl_pin!(PIO4_9, pio4_9, 4, 9);
impl_pin!(PIO4_10, pio4_10, 4, 10);

// GPIO port 7
impl_pin!(PIO7_24, pio7_24, 7, 24);
impl_pin!(PIO7_25, pio7_25, 7, 25);
impl_pin!(PIO7_26, pio7_26, 7, 26);
impl_pin!(PIO7_27, pio7_27, 7, 27);
impl_pin!(PIO7_28, pio7_28, 7, 28);
impl_pin!(PIO7_29, pio7_29, 7, 29);
impl_pin!(PIO7_30, pio7_30, 7, 30);
impl_pin!(PIO7_31, pio7_31, 7, 31);
