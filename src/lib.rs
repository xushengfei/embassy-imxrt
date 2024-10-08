#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

#[cfg(not(any(feature = "mimxrt632s", feature = "mimxrt685s",)))]
compile_error!(
    "No chip feature activated. You must activate exactly one of the following features:
    mimxrt632s,
    mimxrt685s,
    "
);

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

// Disabling ADC for now due to differences in the SVD. Reenable once
// SVD is patched and PAC regenerated.
#[cfg(not(feature = "mimxrt632s"))]
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
pub mod wwdt;

// This mod MUST go last, so that it sees all the `impl_foo!' macros
#[cfg_attr(feature = "mimxrt632s", path = "chips/mimxrt632s.rs")]
#[cfg_attr(feature = "mimxrt685s", path = "chips/mimxrt685s.rs")]
mod chip;

// Reexports
pub use chip::{interrupts::*, peripherals, Peripherals};
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

#[cfg(feature = "unstable-pac")]
pub use chip::pac;

#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;

#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

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
                clocks: ClockConfig::crystal(24_000_000),
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
        clocks::init(config.clocks);
        #[cfg(feature = "time-driver")]
        time_driver::init(config.time_interrupt_priority);
        // dma::init();
    }

    peripherals
}
