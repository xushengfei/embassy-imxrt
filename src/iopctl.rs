//! IO Pad Controller (IOPCTL)
//!
//! Also known as IO Pin Configuration (IOCON)

/// Pin function number.
pub enum Function {
    /// Function 0
    F0,
    /// Function 1
    F1,
    /// Function 2
    F2,
    /// Function 3
    F3,
    /// Function 4
    F4,
    /// Function 5
    F5,
    /// Function 6
    F6,
    /// Function 7
    F7,
    /// Function 8
    F8,
}

/// Internal pull-up/down resistors on a pin.
pub enum Pull {
    /// No pull-up or pull-down resistor selected
    None,
    /// Pull-up resistor
    Up,
    /// Pull-down resistor
    Down,
}

/// Pin slew rate.
pub enum SlewRate {
    /// Standard slew rate
    Standard,
    /// Slow slew rate
    Slow,
}

/// Output drive strength of a pin.
pub enum DriveStrength {
    /// Normal
    Normal,
    /// Full
    Full,
}

/// Output drive mode of a pin.
pub enum DriveMode {
    /// Push-Pull
    PushPull,
    /// Pseudo Open-Drain
    OpenDrain,
}

/// Input polarity of a pin.
pub enum Polarity {
    /// Active-high
    ActiveHigh,
    /// Active-low, which essentially "inverts" the input signal
    ///
    /// e.g. A logic low on an input signal will be interpreted as a logic high
    ActiveLow,
}

trait SealedPin {
    // This is private to prevent users from accessing the register block directly.
    fn regs() -> crate::pac::Iopctl {
        // SAFETY: Through the IOPCTL HAL interface, only a specific register belonging to a specific
        // pin peripheral is accessed at a time. Typically, other peripheral HALs in this crate
        // will consume the pin they are configuring, thus preventing the possibility of the register
        // belonging to that pin being modified simultaneously elsewhere by another peripheral HAL.
        unsafe { crate::pac::Iopctl::steal() }
    }
}

/// A pin.
///
/// This functionality is shared by both
/// "fail-safe" and "high-speed" pins.
#[allow(private_bounds)]
pub trait Pin: SealedPin {
    /// Sets the function number of a pin.
    ///
    /// This number corresponds to a specific function that the pin supports.
    ///
    /// Typically, function 0 corresponds to GPIO while other numbers correspond to a special function.
    ///
    /// See Section 7.5.3 in reference manual for list of pins and their supported functions.
    fn set_function(&self, function: Function) -> &Self;

    /// Enables either a pull-up or pull-down resistor on a pin.
    ///
    /// Setting this to [Pull::None] will disable the resistor.
    fn set_pull(&self, pull: Pull) -> &Self;

    /// Enables the input buffer of a pin.
    ///
    /// This must be enabled for any pin acting as an input,
    /// and some peripheral pins acting as output may need this enabled as well.
    ///
    /// If there is any doubt, it is best to enable the input buffer.
    ///
    /// See Section 7.4.2.3 of reference manual.
    fn enable_input_buffer(&self) -> &Self;

    /// Disables the input buffer of a pin.
    fn disable_input_buffer(&self) -> &Self;

    /// Sets the output drive strength of a pin.
    ///
    /// A drive strength of [DriveStrength::Full] has twice the
    /// high and low drive capability of the [DriveStrength::Normal] setting.
    fn set_drive_strength(&self, strength: DriveStrength) -> &Self;

    /// Sets the ouput drive mode of a pin.
    ///
    /// A pin configured as [DriveMode::OpenDrain] actually operates in
    /// a "pseudo" open-drain mode which is somewhat different than true open-drain.
    ///
    /// See Section 7.4.2.7 of reference manual.
    fn set_drive_mode(&self, mode: DriveMode) -> &Self;

    /// Sets the polarity of an input pin.
    ///
    /// Setting this to [Polarity::ActiveLow] will invert
    /// the input signal.
    fn set_input_polarity(&self, polarity: Polarity) -> &Self;

    /// Returns a pin to its reset state.
    fn reset(&self) -> &Self;
}

/// A fail-safe pin (see Chapter 7 - Pinning Information of datasheet).
///
/// From a software perspective, the only difference is these pins additionally
/// support changing slew rate and enabling analog multiplexing.
///
/// Most pins are "fail-safe" pins. See Table 296 in reference manual
/// for list of pins that are "high-speed" pins and thus don't
/// support this functionality.
pub trait FailSafePin: Pin {
    /// Sets the slew rate of a pin.
    ///
    /// This controls the speed at which a pin can toggle,
    /// which is voltage and load dependent.
    fn set_slew_rate(&self, slew_rate: SlewRate) -> &Self;

    /// Enables the analog multiplexer of a pin.
    ///
    /// This must be called to allow analog functionalities of a pin.
    ///
    /// To protect the analog input, [Pin::set_function] should be
    /// called with [Function::F0] to disable digital functions.
    ///
    /// Additionally, [Pin::disable_input_buffer] and [Pin::set_pull]
    /// with [Pull::None] should be called.
    fn enable_analog_multiplex(&self) -> &Self;

    /// Disables the analog multiplexer of a pin.
    fn disable_analog_multiplex(&self) -> &Self;
}

macro_rules! impl_pin {
    ($pin_periph:ident, $pin_reg:ident) => {
        impl SealedPin for crate::peripherals::$pin_periph {}
        impl Pin for crate::peripherals::$pin_periph {
            fn set_function(&self, function: Function) -> &Self {
                match function {
                    Function::F0 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_0()),
                    Function::F1 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_1()),
                    Function::F2 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_2()),
                    Function::F3 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_3()),
                    Function::F4 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_4()),
                    Function::F5 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_5()),
                    Function::F6 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_6()),
                    Function::F7 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_7()),
                    Function::F8 => Self::regs().$pin_reg().modify(|_, w| w.fsel().function_8()),
                }
                self
            }

            fn set_pull(&self, pull: Pull) -> &Self {
                match pull {
                    Pull::None => Self::regs()
                        .$pin_reg()
                        .modify(|_, w| w.pupdena().disabled().pupdsel().pull_down()),
                    Pull::Up => Self::regs()
                        .$pin_reg()
                        .modify(|_, w| w.pupdena().enabled().pupdsel().pull_up()),
                    Pull::Down => Self::regs()
                        .$pin_reg()
                        .modify(|_, w| w.pupdena().enabled().pupdsel().pull_down()),
                }
                self
            }

            fn enable_input_buffer(&self) -> &Self {
                Self::regs().$pin_reg().modify(|_, w| w.ibena().enabled());
                self
            }

            fn disable_input_buffer(&self) -> &Self {
                Self::regs().$pin_reg().modify(|_, w| w.ibena().disabled());
                self
            }

            fn set_drive_strength(&self, strength: DriveStrength) -> &Self {
                match strength {
                    DriveStrength::Normal => Self::regs()
                        .$pin_reg()
                        .modify(|_, w| w.fulldrive().normal_drive()),
                    DriveStrength::Full => Self::regs().$pin_reg().modify(|_, w| w.fulldrive().full_drive()),
                }
                self
            }

            fn set_drive_mode(&self, mode: DriveMode) -> &Self {
                match mode {
                    DriveMode::PushPull => Self::regs().$pin_reg().modify(|_, w| w.odena().disabled()),
                    DriveMode::OpenDrain => Self::regs().$pin_reg().modify(|_, w| w.odena().enabled()),
                }
                self
            }

            fn set_input_polarity(&self, polarity: Polarity) -> &Self {
                match polarity {
                    Polarity::ActiveHigh => Self::regs().$pin_reg().modify(|_, w| w.iiena().disabled()),
                    Polarity::ActiveLow => Self::regs().$pin_reg().modify(|_, w| w.iiena().enabled()),
                }
                self
            }

            fn reset(&self) -> &Self {
                Self::regs().$pin_reg().reset();
                self
            }
        }
    };
}

macro_rules! impl_failsafe_pin {
    ($pin_periph:ident, $pin_reg:ident) => {
        impl_pin!($pin_periph, $pin_reg);
        impl FailSafePin for crate::peripherals::$pin_periph {
            fn set_slew_rate(&self, slew_rate: SlewRate) -> &Self {
                match slew_rate {
                    SlewRate::Standard => Self::regs().$pin_reg().modify(|_, w| w.slewrate().normal()),
                    SlewRate::Slow => Self::regs().$pin_reg().modify(|_, w| w.slewrate().slow()),
                }
                self
            }

            fn enable_analog_multiplex(&self) -> &Self {
                Self::regs().$pin_reg().modify(|_, w| w.amena().enabled());
                self
            }

            fn disable_analog_multiplex(&self) -> &Self {
                Self::regs().$pin_reg().modify(|_, w| w.amena().disabled());
                self
            }
        }
    };
}

// High-speed pins
impl_pin!(PIO0_21, pio0_21);
impl_pin!(PIO0_22, pio0_22);
impl_pin!(PIO0_23, pio0_23);
impl_pin!(PIO1_18, pio1_18);
impl_pin!(PIO1_19, pio1_19);
impl_pin!(PIO1_20, pio1_20);
impl_pin!(PIO1_21, pio1_21);
impl_pin!(PIO1_22, pio1_22);
impl_pin!(PIO1_23, pio1_23);
impl_pin!(PIO1_24, pio1_24);
impl_pin!(PIO1_25, pio1_25);
impl_pin!(PIO1_26, pio1_26);
impl_pin!(PIO1_27, pio1_27);
impl_pin!(PIO1_28, pio1_28);
impl_pin!(PIO1_29, pio1_29);
impl_pin!(PIO1_30, pio1_30);
impl_pin!(PIO1_31, pio1_31);
impl_pin!(PIO2_0, pio2_0);
impl_pin!(PIO2_1, pio2_1);
impl_pin!(PIO2_2, pio2_2);
impl_pin!(PIO2_3, pio2_3);
impl_pin!(PIO2_4, pio2_4);
impl_pin!(PIO2_5, pio2_5);
impl_pin!(PIO2_6, pio2_6);
impl_pin!(PIO2_7, pio2_7);
impl_pin!(PIO2_8, pio2_8);

// Fail-safe pins
impl_failsafe_pin!(PIO0_0, pio0_0);
impl_failsafe_pin!(PIO0_1, pio0_1);
impl_failsafe_pin!(PIO0_2, pio0_2);
impl_failsafe_pin!(PIO0_3, pio0_3);
impl_failsafe_pin!(PIO0_4, pio0_4);
impl_failsafe_pin!(PIO0_5, pio0_5);
impl_failsafe_pin!(PIO0_6, pio0_6);
impl_failsafe_pin!(PIO0_7, pio0_7);
impl_failsafe_pin!(PIO0_8, pio0_8);
impl_failsafe_pin!(PIO0_9, pio0_9);
impl_failsafe_pin!(PIO0_10, pio0_10);
impl_failsafe_pin!(PIO0_11, pio0_11);
impl_failsafe_pin!(PIO0_12, pio0_12);
impl_failsafe_pin!(PIO0_13, pio0_13);
impl_failsafe_pin!(PIO0_14, pio0_14);
impl_failsafe_pin!(PIO0_15, pio0_15);
impl_failsafe_pin!(PIO0_16, pio0_16);
impl_failsafe_pin!(PIO0_17, pio0_17);
impl_failsafe_pin!(PIO0_18, pio0_18);
impl_failsafe_pin!(PIO0_19, pio0_19);
impl_failsafe_pin!(PIO0_20, pio0_20);
impl_failsafe_pin!(PIO0_24, pio0_24);
impl_failsafe_pin!(PIO0_25, pio0_25);
impl_failsafe_pin!(PIO0_26, pio0_26);
impl_failsafe_pin!(PIO0_27, pio0_27);
impl_failsafe_pin!(PIO0_28, pio0_28);
impl_failsafe_pin!(PIO0_29, pio0_29);
impl_failsafe_pin!(PIO0_30, pio0_30);
impl_failsafe_pin!(PIO0_31, pio0_31);
impl_failsafe_pin!(PIO1_0, pio1_0);
impl_failsafe_pin!(PIO1_1, pio1_1);
impl_failsafe_pin!(PIO1_2, pio1_2);
impl_failsafe_pin!(PIO1_3, pio1_3);
impl_failsafe_pin!(PIO1_4, pio1_4);
impl_failsafe_pin!(PIO1_5, pio1_5);
impl_failsafe_pin!(PIO1_6, pio1_6);
impl_failsafe_pin!(PIO1_7, pio1_7);
impl_failsafe_pin!(PIO1_8, pio1_8);
impl_failsafe_pin!(PIO1_9, pio1_9);
impl_failsafe_pin!(PIO1_10, pio1_10);
impl_failsafe_pin!(PIO1_11, pio1_11);
impl_failsafe_pin!(PIO1_12, pio1_12);
impl_failsafe_pin!(PIO1_13, pio1_13);
impl_failsafe_pin!(PIO1_14, pio1_14);
impl_failsafe_pin!(PIO1_15, pio1_15);
impl_failsafe_pin!(PIO1_16, pio1_16);
impl_failsafe_pin!(PIO1_17, pio1_17);
impl_failsafe_pin!(PIO2_9, pio2_9);
impl_failsafe_pin!(PIO2_10, pio2_10);
impl_failsafe_pin!(PIO2_11, pio2_11);
impl_failsafe_pin!(PIO2_12, pio2_12);
impl_failsafe_pin!(PIO2_13, pio2_13);
impl_failsafe_pin!(PIO2_14, pio2_14);
impl_failsafe_pin!(PIO2_15, pio2_15);
impl_failsafe_pin!(PIO2_16, pio2_16);
impl_failsafe_pin!(PIO2_17, pio2_17);
impl_failsafe_pin!(PIO2_18, pio2_18);
impl_failsafe_pin!(PIO2_19, pio2_19);
impl_failsafe_pin!(PIO2_20, pio2_20);
impl_failsafe_pin!(PIO2_21, pio2_21);
impl_failsafe_pin!(PIO2_22, pio2_22);
impl_failsafe_pin!(PIO2_23, pio2_23);
impl_failsafe_pin!(PIO2_24, pio2_24);

// Note: These have have reset values of 0x41 to support SWD by default
impl_failsafe_pin!(PIO2_25, pio2_25);
impl_failsafe_pin!(PIO2_26, pio2_26);

impl_failsafe_pin!(PIO2_27, pio2_27);
impl_failsafe_pin!(PIO2_28, pio2_28);
impl_failsafe_pin!(PIO2_29, pio2_29);
impl_failsafe_pin!(PIO2_30, pio2_30);
impl_failsafe_pin!(PIO2_31, pio2_31);
impl_failsafe_pin!(PIO3_0, pio3_0);
impl_failsafe_pin!(PIO3_1, pio3_1);
impl_failsafe_pin!(PIO3_2, pio3_2);
impl_failsafe_pin!(PIO3_3, pio3_3);
impl_failsafe_pin!(PIO3_4, pio3_4);
impl_failsafe_pin!(PIO3_5, pio3_5);
impl_failsafe_pin!(PIO3_6, pio3_6);
impl_failsafe_pin!(PIO3_7, pio3_7);
impl_failsafe_pin!(PIO3_8, pio3_8);
impl_failsafe_pin!(PIO3_9, pio3_9);
impl_failsafe_pin!(PIO3_10, pio3_10);
impl_failsafe_pin!(PIO3_11, pio3_11);
impl_failsafe_pin!(PIO3_12, pio3_12);
impl_failsafe_pin!(PIO3_13, pio3_13);
impl_failsafe_pin!(PIO3_14, pio3_14);
impl_failsafe_pin!(PIO3_15, pio3_15);
impl_failsafe_pin!(PIO3_16, pio3_16);
impl_failsafe_pin!(PIO3_17, pio3_17);
impl_failsafe_pin!(PIO3_18, pio3_18);
impl_failsafe_pin!(PIO3_19, pio3_19);
impl_failsafe_pin!(PIO3_20, pio3_20);
impl_failsafe_pin!(PIO3_21, pio3_21);
impl_failsafe_pin!(PIO3_22, pio3_22);
impl_failsafe_pin!(PIO3_23, pio3_23);
impl_failsafe_pin!(PIO3_24, pio3_24);
impl_failsafe_pin!(PIO3_25, pio3_25);
impl_failsafe_pin!(PIO3_26, pio3_26);
impl_failsafe_pin!(PIO3_27, pio3_27);
impl_failsafe_pin!(PIO3_28, pio3_28);
impl_failsafe_pin!(PIO3_29, pio3_29);
impl_failsafe_pin!(PIO3_30, pio3_30);
impl_failsafe_pin!(PIO3_31, pio3_31);
impl_failsafe_pin!(PIO4_0, pio4_0);
impl_failsafe_pin!(PIO4_1, pio4_1);
impl_failsafe_pin!(PIO4_2, pio4_2);
impl_failsafe_pin!(PIO4_3, pio4_3);
impl_failsafe_pin!(PIO4_4, pio4_4);
impl_failsafe_pin!(PIO4_5, pio4_5);
impl_failsafe_pin!(PIO4_6, pio4_6);
impl_failsafe_pin!(PIO4_7, pio4_7);
impl_failsafe_pin!(PIO4_8, pio4_8);
impl_failsafe_pin!(PIO4_9, pio4_9);
impl_failsafe_pin!(PIO4_10, pio4_10);
impl_failsafe_pin!(PIO7_24, pio7_24);
impl_failsafe_pin!(PIO7_25, pio7_25);
impl_failsafe_pin!(PIO7_26, pio7_26);
impl_failsafe_pin!(PIO7_27, pio7_27);
impl_failsafe_pin!(PIO7_28, pio7_28);
impl_failsafe_pin!(PIO7_29, pio7_29);
impl_failsafe_pin!(PIO7_30, pio7_30);
impl_failsafe_pin!(PIO7_31, pio7_31);
