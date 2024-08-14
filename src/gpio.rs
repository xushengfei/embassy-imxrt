//! GPIO driver.
#![macro_use]
use core::convert::Infallible;
use core::hint::unreachable_unchecked;

use embassy_hal_internal::{impl_peripheral, into_ref, PeripheralRef};

use crate::{pac, peripherals, Peripheral};

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

/// Pull setting for an input.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// Internal pull-up resistor.
    Up,

    /// Internal pull-down resistor.
    Down,
}

/// GPIO input driver.
pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create GPIO Input driver for a [Pin] with the provided [Level] configuration.
    pub fn new(pin: impl Pin + 'd, pull: Pull) -> Self {
        pin.set_as_input(pull);
        let mut pin = Flex::new(pin);
        pin.set_as_input();

        Self { pin }
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }
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

// TODO: Match these drive strenghts to what RT600 manual describes
/// Drive strength settings for an output pin.
// These numbers match DRIVE_A exactly so hopefully the compiler will unify them.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputDrive {
    /// Normal drive strength.
    Normal,
    /// Full drive strength. Twice that of Normal drive strength.
    Full,
}

/// GPIO output drivber.
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}
impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] configuration.
    #[inline]
    pub fn new(pin: impl Pin + 'd, initial_output: Level, drive: OutputDrive) -> Self {
        pin.set_as_output(drive); // setting drive

        let mut pin = Flex::new(pin);
        pin.set_as_output();

        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
    }

    /// Get whether the output level is set to high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Get whether the output level is set to low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.pin.get_output_level()
    }
}

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin, or both. The level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d> {
    pub(crate) pin: PeripheralRef<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    #[inline]
    pub fn new(pin: impl Pin + 'd) -> Self {
        into_ref!(pin);
        // Pin will be in disconnected state.
        Self { pin: pin.map_into() }
    }

    /// Put the pin into input mode.
    pub fn set_as_input(&mut self) {
        let port = (self.pin.pin_port() as usize) / 32;
        let pin = self.pin.pin() as usize;
        self.pin
            .block()
            .dir(port)
            .modify(|r, w| unsafe { w.dirp().bits(r.dirp().bits() & !(1 << pin)) });
    }

    /// Get whether the input level is set to high.
    #[inline]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    /// Get whether the input level is set to low.
    #[inline]
    pub fn is_low(&self) -> bool {
        let port = (self.pin.pin_port() as usize) / 32;
        let pin = self.pin.pin() as usize;
        let bits = self.pin.block().set(port).read().bits();
        bits & (1 << pin) == 0
    }

    /// Get the current input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        let port = (self.pin.pin_port() as usize) / 32;
        let pin = self.pin.pin() as usize;
        self.pin
            .block()
            .dir(port)
            .modify(|r, w| unsafe { w.dirp().bits(r.dirp().bits() | (1 << pin)) });
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        // use toggle register -- why?
        let port = (self.pin.pin_port() as usize) / 32;
        let pin = self.pin.pin() as usize;
        self.pin.block().not(port).write(|w| unsafe { w.bits(1 << pin) });
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.pin.set_low(),
            Level::High => self.pin.set_high(),
        }
    }

    /// Get whether the output level is set to high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Get whether the output level is set to low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        let port = (self.pin.pin_port() as usize) / 32;
        let pin = self.pin.pin() as usize;
        let bits = self.pin.block().set(port).read().bits();
        bits & (1 << pin) == 0
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Put the pin into input + output mode.
    ///
    /// This is commonly used for "open drain" mode. If you set `drive = Standard0Disconnect1`,
    /// the hardware will drive the line low if you set it to low, and will leave it floating if you set
    /// it to high, in which case you can read the input to figure out whether another device
    /// is driving the line low.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_input_output(&mut self, _pull: Pull, _drive: OutputDrive) {
        todo!()
    }

    /// Put the pin into disconnected mode.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        todo!()
    }
}

impl<'d> Drop for Flex<'d> {
    fn drop(&mut self) {
        // bring pin back to reset state
        todo!()
    }
}

trait SealedPin {
    fn pin_port(&self) -> u8;

    #[inline]
    fn _pin(&self) -> u8 {
        self.pin_port() % 32
    }

    #[inline]
    fn block(&self) -> &pac::gpio::RegisterBlock {
        unsafe { &*pac::Gpio::ptr() }
    }

    /// Set the output as high.
    #[inline]
    fn set_high(&self) {
        let port = (self.pin_port() as usize) / 32;
        let pin = self._pin() as usize;
        self.block().set(port).write(|w| unsafe { w.bits(1 << pin) });
    }

    /// Set the output as low.
    #[inline]
    fn set_low(&self) {
        let port = (self.pin_port() as usize) / 32;
        let pin = self._pin() as usize;
        self.block().clr(port).write(|w| unsafe { w.bits(1 << pin) });
    }
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an [AnyPin].
#[allow(private_bounds)]
pub trait Pin: Peripheral<P = Self> + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Number of the pin within the port (0..31)
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    /// Port of the pin
    #[inline]
    fn port(&self) -> Port {
        match (self.pin_port() as usize) / 32 {
            0 => Port::Port0,
            1 => Port::Port1,
            2 => Port::Port2,
            3 => Port::Port3,
            4 => Port::Port4,
            7 => Port::Port7,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// To set Input Pin with [Pull] as Up/Down
    fn set_as_input(&self, pull: Pull);

    /// To set Output Pin with [OutputDrive] as full_drive/normal_drive
    fn set_as_output(&self, drive: OutputDrive);

    /// Convert from concrete pin type PX_XX to type erased `AnyPin`.
    #[inline]
    fn degrade(self) -> AnyPin {
        AnyPin {
            pin_port: self.pin_port(),
        }
    }
}

/// Type-erased GPIO pin
pub struct AnyPin {
    pin_port: u8,
}

impl AnyPin {
    /// Create an [AnyPin] for a specific pin.
    ///
    /// # Safety
    /// should not be in use by another driver.
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Self {
        Self { pin_port }
    }
}

impl_peripheral!(AnyPin);
impl Pin for AnyPin {
    fn set_as_output(&self, _drive: OutputDrive) {
        let port = (self.pin_port() as usize) / 32;
        let pin = self.pin() as usize;
        self.block()
            .dir(port)
            .modify(|r, w| unsafe { w.dirp().bits(r.dirp().bits() | (1 << pin)) });
    }
    fn set_as_input(&self, _pull: Pull) {
        let port = (self.pin_port() as usize) / 32;
        let pin = self.pin() as usize;
        self.block()
            .dir(port)
            .modify(|r, w| unsafe { w.dirp().bits(r.dirp().bits() & !(1 << pin)) });
    }
}

impl SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

// ====================

mod eh02 {
    use super::*;

    impl<'d> embedded_hal_02::digital::v2::InputPin for Input<'d> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::OutputPin for Output<'d> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for Output<'d> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for Output<'d> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.toggle();
            Ok(())
        }
    }

    /// Implement [`embedded_hal_02::digital::v2::InputPin`] for [`Flex`];
    ///
    /// If the pin is not in input mode the result is unspecified.
    impl<'d> embedded_hal_02::digital::v2::InputPin for Flex<'d> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::OutputPin for Flex<'d> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'d> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'d> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.toggle();
            Ok(())
        }
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Input<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::InputPin for Input<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::OutputPin for Output<'d> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for Output<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Flex<'d> {
    type Error = Infallible;
}

/// Implement [`InputPin`] for [`Flex`];
///
/// If the pin is not in input mode the result is unspecified.
impl<'d> embedded_hal_1::digital::InputPin for Flex<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::OutputPin for Flex<'d> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for Flex<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

/// Enables each GPIO port 0..7
pub fn init() {
    // Enable GPIO clocks
    let r = unsafe { &*(pac::Clkctl1::ptr()) };
    r.pscctl1_set().write(|w| w.hsgpio0_clk_set().set_clock());
    r.pscctl1_set().write(|w| w.hsgpio1_clk_set().set_clock());
    r.pscctl1_set().write(|w| w.hsgpio2_clk_set().set_clock());
    r.pscctl1_set().write(|w| w.hsgpio3_clk_set().set_clock());
    r.pscctl1_set().write(|w| w.hsgpio4_clk_set().set_clock());
    r.pscctl1_set().write(|w| w.hsgpio7_clk_set().set_clock());
    // Take GPIO out of reset
    let r = unsafe { &*(pac::Rstctl1::ptr()) };
    r.prstctl1_clr().write(|w| w.hsgpio0_rst_clr().clr_reset());
    r.prstctl1_clr().write(|w| w.hsgpio1_rst_clr().clr_reset());
    r.prstctl1_clr().write(|w| w.hsgpio2_rst_clr().clr_reset());
    r.prstctl1_clr().write(|w| w.hsgpio3_rst_clr().clr_reset());
    r.prstctl1_clr().write(|w| w.hsgpio4_rst_clr().clr_reset());
    r.prstctl1_clr().write(|w| w.hsgpio7_rst_clr().clr_reset());
}

macro_rules! impl_pin {
    ($peripheral:ident, $method:ident, $port_num:expr, $pin_num:expr) => {
        impl crate::gpio::Pin for peripherals::$peripheral {
            #[inline]
            fn set_as_output(&self, drive: OutputDrive) {
                let iopctl_ptr = unsafe { &*crate::pac::Iopctl::ptr() };
                iopctl_ptr.$method().write(|w| {
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
                        .amena()
                        .disabled()
                        .odena()
                        .disabled()
                        .iiena()
                        .disabled()
                });
                match drive {
                    OutputDrive::Normal => iopctl_ptr.$method().modify(|_, w| w.fulldrive().normal_drive()),
                    OutputDrive::Full => iopctl_ptr.$method().modify(|_, w| w.fulldrive().full_drive()),
                }
            }

            #[inline]
            fn set_as_input(&self, pull: Pull) {
                let iopctl_ptr = unsafe { &*crate::pac::Iopctl::ptr() };
                iopctl_ptr.$method().write(|w| {
                    w.fsel()
                        .function_0()
                        .pupdena()
                        .enabled()
                        .ibena()
                        .enabled()
                        .slewrate()
                        .normal()
                        .amena()
                        .disabled()
                        .odena()
                        .disabled()
                        .iiena()
                        .enabled()
                });
                match pull {
                    Pull::Up => iopctl_ptr.$method().modify(|_, w| w.pupdsel().pull_up()),
                    Pull::Down => iopctl_ptr.$method().modify(|_, w| w.pupdsel().pull_down()),
                }
            }
        }

        impl crate::gpio::SealedPin for peripherals::$peripheral {
            #[inline]
            fn pin_port(&self) -> u8 {
                $port_num * 32 + $pin_num
            }
        }

        impl From<peripherals::$peripheral> for crate::gpio::AnyPin {
            fn from(val: peripherals::$peripheral) -> Self {
                crate::gpio::Pin::degrade(val)
            }
        }
    };
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
