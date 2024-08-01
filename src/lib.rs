#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

// This mod MUST go first, so that the others see its macros.
// pub(crate) mod fmt;

pub mod adc;
pub mod clocks;
#[cfg(feature = "time-driver")]
mod time_driver;
pub mod wwdt;

// Reexports
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
pub use mimxrt685s_pac as pac;

#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

embassy_hal_internal::interrupt_mod!(
    WDT0,
    DMA0,
    GPIO_INTA,
    GPIO_INTB,
    PIN_INT0,
    PIN_INT1,
    PIN_INT2,
    PIN_INT3,
    UTICK0,
    MRT0,
    CTIMER0,
    CTIMER1,
    SCT0,
    CTIMER3,
    FLEXCOMM0,
    FLEXCOMM1,
    FLEXCOMM2,
    FLEXCOMM3,
    FLEXCOMM4,
    FLEXCOMM5,
    FLEXCOMM14,
    FLEXCOMM15,
    ADC0,
    ACMP,
    DMIC0,
    HYPERVISOR,
    SECUREVIOLATION,
    HWVAD0,
    RNG,
    RTC,
    DSPWAKE,
    MU_A,
    PIN_INT4,
    PIN_INT5,
    PIN_INT6,
    PIN_INT7,
    CTIMER2,
    CTIMER4,
    OS_EVENT,
    FLEXSPI,
    FLEXCOMM6,
    FLEXCOMM7,
    USDHC0,
    USDHC1,
    SGPIO_INTA,
    SGPIO_INTB,
    I3C0,
    USB,
    USB_WAKEUP,
    WDT1,
    USBPHY_DCD,
    DMA1,
    PUF,
    POWERQUAD,
    CASPER,
    PMC_PMIC,
    HASHCRYPT,
);

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
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
    WDT0,
    DMA0,
    GPIO_INTA,
    GPIO_INTB,
    PIN_INT0,
    PIN_INT1,
    PIN_INT2,
    PIN_INT3,
    UTICK0,
    MRT0,
    CTIMER0,
    CTIMER1,
    SCT0,
    CTIMER3,
    FLEXCOMM0,
    FLEXCOMM1,
    FLEXCOMM2,
    FLEXCOMM3,
    FLEXCOMM4,
    FLEXCOMM5,
    FLEXCOMM14,
    FLEXCOMM15,
    ADC0,
    ACMP,
    DMIC0,
    HYPERVISOR,
    SECUREVIOLATION,
    HWVAD0,
    RNG,
    RTC,
    DSPWAKE,
    MU_A,
    PIN_INT4,
    PIN_INT5,
    PIN_INT6,
    PIN_INT7,
    CTIMER2,
    CTIMER4,
    OS_EVENT,
    FLEXSPI,
    FLEXCOMM6,
    FLEXCOMM7,
    USDHC0,
    USDHC1,
    SGPIO_INTA,
    SGPIO_INTB,
    I3C0,
    USB,
    USB_WAKEUP,
    WDT1,
    USBPHY_DCD,
    DMA1,
    PUF,
    POWERQUAD,
    CASPER,
    PMC_PMIC,
    HASHCRYPT,
);

/// HAL configuration for iMX RT600.
pub mod config {
    use crate::clocks::ClockConfig;

    /// HAL configuration passed when initializing.
    #[non_exhaustive]
    pub struct Config {
        /// Clock configuration.
        pub clocks: ClockConfig,
        /// GPIOTE interrupt priority. Should be lower priority than softdevice if used.
        #[cfg(feature = "gpiote")]
        pub gpiote_interrupt_priority: crate::interrupt::Priority,
        /// Time driver interrupt priority. Should be lower priority than softdevice if used.
        #[cfg(feature = "time-driver")]
        pub time_interrupt_priority: crate::interrupt::Priority,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                clocks: ClockConfig::crystal(24_000_000),
                #[cfg(feature = "gpiote")]
                gpiote_interrupt_priority: crate::interrupt::Priority::P0,
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
                #[cfg(feature = "gpiote")]
                gpiote_interrupt_priority: crate::interrupt::Priority::P0,
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
        clocks::init(config.clocks);
        #[cfg(feature = "time-driver")]
        time_driver::init(config.time_interrupt_priority);
        // dma::init();
        // gpio::init();
    }

    peripherals
}
