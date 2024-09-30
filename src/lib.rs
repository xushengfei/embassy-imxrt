#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod adc;
pub mod clocks;
pub mod crc;
pub mod flexcomm;
pub mod gpio;
pub mod i2c;
pub mod iopctl;
pub mod pwm;
pub mod rng;
#[cfg(feature = "time-driver")]
mod time_driver;
pub mod uart;
pub mod wwdt;

// Reexports
pub use adc::AdcChannel;
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
pub use interrupts::*;
pub use mimxrt685s_pac as pac;

#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

/// Wrapper module to suppress clippy warning caused by macro.
#[allow(clippy::missing_safety_doc)]
pub mod interrupts {
    embassy_hal_internal::interrupt_mod!(
        ACMP,
        ADC0,
        CASPER,
        CTIMER0,
        CTIMER1,
        CTIMER2,
        CTIMER3,
        CTIMER4,
        DMA0,
        DMA1,
        DMIC0,
        DSPWAKE,
        FLEXCOMM0,
        FLEXCOMM1,
        FLEXCOMM14,
        FLEXCOMM15,
        FLEXCOMM2,
        FLEXCOMM3,
        FLEXCOMM4,
        FLEXCOMM5,
        FLEXCOMM6,
        FLEXCOMM7,
        FLEXSPI,
        GPIO_INTA,
        GPIO_INTB,
        HASHCRYPT,
        HWVAD0,
        HYPERVISOR,
        I3C0,
        MRT0,
        MU_A,
        OS_EVENT,
        PIN_INT0,
        PIN_INT1,
        PIN_INT2,
        PIN_INT3,
        PIN_INT4,
        PIN_INT5,
        PIN_INT6,
        PIN_INT7,
        PMC_PMIC,
        POWERQUAD,
        PUF,
        RNG,
        RTC,
        SCT0,
        SECUREVIOLATION,
        SGPIO_INTA,
        SGPIO_INTB,
        USB,
        USBPHY_DCD,
        USB_WAKEUP,
        USDHC0,
        USDHC1,
        UTICK0,
        WDT0,
        WDT1,
    );
}

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right \[`Binding`\]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_imxrt::{bind_interrupts, flexspi, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     FLEXSPI_IRQ => flexspi::InterruptHandler<peripherals::FLEXSPI>;
/// });
/// ```
///
// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident { $($irq:ident => $($handler:ty),*;)* }) => {
            #[derive(Copy, Clone)]
            $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            unsafe extern "C" fn $irq() {
                $(
                    <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();
                )*
            }

            $(
                unsafe impl $crate::interrupt::typelevel::Binding<$crate::interrupt::typelevel::$irq, $handler> for $name {}
            )*
        )*
    };
}

embassy_hal_internal::peripherals!(
    ACMP,
    ADC0,
    CASPER,
    CRC,
    CTIMER0,
    CTIMER1,
    CTIMER2,
    CTIMER3,
    CTIMER4,
    DMA0,
    DMA1,
    DMIC0,
    DSPWAKE,
    FLEXCOMM0,
    FLEXCOMM1,
    FLEXCOMM14,
    FLEXCOMM15,
    FLEXCOMM2,
    FLEXCOMM3,
    FLEXCOMM4,
    FLEXCOMM5,
    FLEXCOMM6,
    FLEXCOMM7,
    FLEXSPI,
    GPIO_INTA,
    GPIO_INTB,
    HASHCRYPT,
    HWVAD0,
    HYPERVISOR,
    I3C0,
    MRT0,
    MU_A,
    OS_EVENT,
    PIN_INT0,
    PIN_INT1,
    PIN_INT2,
    PIN_INT3,
    PIN_INT4,
    PIN_INT5,
    PIN_INT6,
    PIN_INT7,
    PIO0_0,
    PIO0_1,
    PIO0_10,
    PIO0_11,
    PIO0_12,
    PIO0_13,
    PIO0_14,
    PIO0_15,
    PIO0_16,
    PIO0_17,
    PIO0_18,
    PIO0_19,
    PIO0_2,
    PIO0_20,
    PIO0_21,
    PIO0_22,
    PIO0_23,
    PIO0_24,
    PIO0_25,
    PIO0_26,
    PIO0_27,
    PIO0_28,
    PIO0_29,
    PIO0_3,
    PIO0_30,
    PIO0_31,
    PIO0_4,
    PIO0_5,
    PIO0_6,
    PIO0_7,
    PIO0_8,
    PIO0_9,
    PIO1_0,
    PIO1_1,
    PIO1_10,
    PIO1_11,
    PIO1_12,
    PIO1_13,
    PIO1_14,
    PIO1_15,
    PIO1_16,
    PIO1_17,
    PIO1_18,
    PIO1_19,
    PIO1_2,
    PIO1_20,
    PIO1_21,
    PIO1_22,
    PIO1_23,
    PIO1_24,
    PIO1_25,
    PIO1_26,
    PIO1_27,
    PIO1_28,
    PIO1_29,
    PIO1_3,
    PIO1_30,
    PIO1_31,
    PIO1_4,
    PIO1_5,
    PIO1_6,
    PIO1_7,
    PIO1_8,
    PIO1_9,
    PIO2_0,
    PIO2_1,
    PIO2_10,
    PIO2_11,
    PIO2_12,
    PIO2_13,
    PIO2_14,
    PIO2_15,
    PIO2_16,
    PIO2_17,
    PIO2_18,
    PIO2_19,
    PIO2_2,
    PIO2_20,
    PIO2_21,
    PIO2_22,
    PIO2_23,
    PIO2_24,
    PIO2_25,
    PIO2_26,
    PIO2_27,
    PIO2_28,
    PIO2_29,
    PIO2_3,
    PIO2_30,
    PIO2_31,
    PIO2_4,
    PIO2_5,
    PIO2_6,
    PIO2_7,
    PIO2_8,
    PIO2_9,
    PIO3_0,
    PIO3_1,
    PIO3_10,
    PIO3_11,
    PIO3_12,
    PIO3_13,
    PIO3_14,
    PIO3_15,
    PIO3_16,
    PIO3_17,
    PIO3_18,
    PIO3_19,
    PIO3_2,
    PIO3_20,
    PIO3_21,
    PIO3_22,
    PIO3_23,
    PIO3_24,
    PIO3_25,
    PIO3_26,
    PIO3_27,
    PIO3_28,
    PIO3_29,
    PIO3_3,
    PIO3_30,
    PIO3_31,
    PIO3_4,
    PIO3_5,
    PIO3_6,
    PIO3_7,
    PIO3_8,
    PIO3_9,
    PIO4_0,
    PIO4_1,
    PIO4_10,
    PIO4_2,
    PIO4_3,
    PIO4_4,
    PIO4_5,
    PIO4_6,
    PIO4_7,
    PIO4_8,
    PIO4_9,
    PIO7_24,
    PIO7_25,
    PIO7_26,
    PIO7_27,
    PIO7_28,
    PIO7_29,
    PIO7_30,
    PIO7_31,
    PMC_PMIC,
    POWERQUAD,
    PUF,
    RNG,
    RTC,
    SCT0,
    SECUREVIOLATION,
    SGPIO_INTA,
    SGPIO_INTB,
    USB,
    USBPHY_DCD,
    USB_WAKEUP,
    USDHC0,
    USDHC1,
    UTICK0,
    WDT0,
    WDT1,
);

/// HAL configuration for iMX RT600.
pub mod config {
    use crate::clocks::ClockConfig;

    /// HAL configuration passed when initializing.
    #[non_exhaustive]
    pub struct Config {
        /// Clock configuration.
        pub clocks: ClockConfig,
        /// Time driver interrupt priority. Should be lower priority than softdevice if used.
        #[cfg(feature = "time-driver")]
        pub time_interrupt_priority: crate::interrupt::Priority,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                clocks: ClockConfig::crystal(),
                #[cfg(feature = "time-driver")]
                time_interrupt_priority: crate::interrupt::Priority::P0,
            }
        }
    }

    impl Config {
        /// Create a new configuration with the provided clock config.
        pub fn new(clocks: ClockConfig) -> Self {
            Self {
                clocks,
                #[cfg(feature = "time-driver")]
                time_interrupt_priority: crate::interrupt::Priority::P0,
            }
        }
    }
}

/// Initialize the `embassy-imxrt` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once at startup, otherwise it panics.
pub fn init(config: config::Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    unsafe {
        if let Err(e) = clocks::init(config.clocks) {
            error!("unable to initialize Clocks for reason: {:?}", e);
            // Panic here?
        }
        #[cfg(feature = "time-driver")]
        time_driver::init(config.time_interrupt_priority);
        // dma::init();
    }

    peripherals
}
