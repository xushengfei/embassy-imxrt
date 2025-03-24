//! ADC

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::Deref;
use core::task::Poll;

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::enable_and_reset;
use crate::interrupt::typelevel::Binding;
use crate::iopctl::{
    AnyPin, DriveMode, DriveStrength, Function, Inverter, IopctlFunctionPin, IopctlPin, Pull, SlewRate,
};
use crate::pac::adc0;
use crate::peripherals::ADC0;
use crate::{interrupt, peripherals};

static WAKER: AtomicWaker = AtomicWaker::new();

/// ADC error
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Invalid ADC configuration
    InvalidConfig,
}

/// ADC config
pub struct Config {
    /// ADC voltage reference
    pub vref: Reference,
}

impl Default for Config {
    /// Default configuration for single channel sampling.
    fn default() -> Self {
        Self {
            vref: Reference::VddaAdc1v8,
        }
    }
}

/// ADC channel config
pub struct ChannelConfig<'d> {
    /// Positive channel to sample
    p_channel: GuardedAnyInput<'d>,
    /// An optional negative channel to sample
    n_channel: Option<GuardedAnyInput<'d>>,
}

impl<'d> ChannelConfig<'d> {
    /// Default configuration for single ended channel sampling.
    pub fn single_ended(input: impl Peripheral<P = impl Input> + 'd) -> Self {
        into_ref!(input);
        Self {
            p_channel: input.map_into().into(),
            n_channel: None,
        }
    }
    /// Default configuration for differential channel sampling.
    pub fn differential(
        p_input: impl Peripheral<P = impl Input> + 'd,
        n_input: impl Peripheral<P = impl Input> + 'd,
    ) -> Result<Self, Error> {
        into_ref!(p_input, n_input);

        let p: PeripheralRef<'_, AnyInput> = p_input.map_into();
        let n: PeripheralRef<'_, AnyInput> = n_input.map_into();

        // Check matching positive and negative pin are passed in
        // Do not need to check for side as there are only 1 channel for each
        //   polarity
        if p.channel().ch != n.channel().ch {
            return Err(Error::InvalidConfig);
        }

        Ok(Self {
            p_channel: p.into(),
            n_channel: Some(n.into()),
        })
    }
}

/// ADC interrupt handler
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let reg = T::info().regs;

        // Disable fifo watermark interrupt
        reg.ie().write(|w| w.fwmie().fwmie_0());
        WAKER.wake();
    }
}

/// ADC driver
pub struct Adc<'p, const N: usize> {
    info: Info,
    _lifetime: PhantomData<&'p ()>,
}

struct Info {
    regs: crate::pac::Adc0,
}

impl<const N: usize> Adc<'_, N> {
    fn init() {
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

        // Power up ADC block
        sysctl0
            .pdruncfg0_clr()
            .write(|w| w.adc_pd().set_bit().adc_lp().set_bit());

        // Configure ADC clock mux
        // Select LPOSC for now, unless we want to speed up the clocks
        clkctl0.adc0fclksel0().write(|w| w.sel().lposc());
        clkctl0.adc0fclksel1().write(|w| w.sel().adc0fclksel0_mux_out());

        // Set ADC clock divisor
        clkctl0.adc0fclkdiv().modify(|_, w| w.reset().set_bit());
        clkctl0
            .adc0fclkdiv()
            .write(|w| unsafe { w.div().bits(0x0).halt().clear_bit() });
        while clkctl0.adc0fclkdiv().read().reqflag().bit_is_set() {}

        enable_and_reset::<ADC0>();
    }

    fn configure_adc(&mut self, config: Config) {
        // Reset ADC
        self.info.regs.ctrl().modify(|_, w| w.rst().rst_1());
        self.info.regs.ctrl().modify(|_, w| w.rst().rst_0());

        // Reset ADC fifo
        self.info.regs.ctrl().modify(|_, w| w.rstfifo().rstfifo_1());

        // Disable ADC before configuration
        self.info.regs.ctrl().modify(|_, w| w.adcen().adcen_0());

        // Disable ADC in doze Mode
        self.info.regs.ctrl().modify(|_, w| w.dozen().dozen_1());

        // Configure ADC
        self.info.regs.cfg().write(|w| unsafe {
            w.tprictrl()
                .tprictrl_1() /* Allow current conversion to finish */
                /* even if a higher priority trigger is received */
                .pwrsel()
                .pwrsel_3() /* Highest power mode */
                .refsel()
                .variant(config.vref.into()) /* Voltage reference */
                .pudly()
                .bits(0x00) /* No power up delay */
                .pwren()
                .pwren_1() /* Pre-energize the analog circuit */
        });

        // No pause delay between conversion
        self.info.regs.pause().write(|w| w.pauseen().pauseen_0());

        // Re-enable ADC after configuration
        self.info.regs.ctrl().modify(|_, w| w.adcen().adcen_1());

        // Reset ADC fifo
        self.info.regs.ctrl().modify(|_, w| w.rstfifo().rstfifo_1());
    }

    fn configure_channels(&mut self, channel_config: &[ChannelConfig; N]) {
        let mut cmd = channel_config.len();

        // Configure conversion CMD configuration
        // Set up a cmd chain, one cmd per channel
        //   one points to the next, last one points to 0
        for ch in channel_config {
            // Mapping cmd [1-15] into reg array index [0-14]
            // Reg array index is one less than cmd
            let cmd_index = cmd - 1;
            let p = ch.p_channel.channel();
            let diff = match ch.n_channel {
                None => adc0::cmdl::Diff::Diff0,
                Some(_) => adc0::cmdl::Diff::Diff1,
            };

            self.info.regs.cmdl(cmd_index).write(|w| {
                w.adch()
                    .variant(p.ch) /* Analog channel number */
                    .absel()
                    .variant(p.side.into()) /* A/B side select */
                    .diff()
                    .variant(diff) /* Differential or single-ended */
                    .cscale()
                    .cscale_1() /* Full scale */
            });

            self.info.regs.cmdh(cmd_index).write(|w| unsafe {
                w.cmpen()
                    .cmpen_0() /* Disable analog comparator */
                    .lwi()
                    .clear_bit() /* Disable auto channel auto increment */
                    .sts()
                    .sts_7()
                    .avgs()
                    .avgs_0()
                    .loop_()
                    .loop_0()
                    .next()
                    .bits((cmd - 1) as u8)
            });

            // Shift to next cmd-channel pair
            cmd -= 1;
        }

        /* Set trigger configuration. */
        self.info.regs.tctrl(0).write(|w| unsafe {
            w.hten()
                .clear_bit()
                .tpri()
                .tpri_0()
                .tdly()
                .bits(0)
                .tcmd()
                .bits(channel_config.len() as u8)
        });
    }
}

impl<'p, const N: usize> Adc<'p, N> {
    /// Create ADC driver.
    pub fn new<T: Instance>(
        _adc: impl Peripheral<P = T> + 'p,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'p,
        config: Config,
        channel_config: [ChannelConfig; N],
    ) -> Self {
        into_ref!(_adc);

        let mut inst = Self {
            info: T::info(),
            _lifetime: PhantomData,
        };

        Self::init();
        inst.configure_adc(config);
        inst.configure_channels(&channel_config);

        // Enable interrupt
        interrupt::ADC0.unpend();
        unsafe { interrupt::ADC0.enable() };

        inst
    }

    /// One shot sampling. The buffer must be the same size as the number of channels configured.
    /// The sampling is stopped prior to returning in order to reduce power consumption (power
    /// consumption remains higher if sampling is not stopped explicitly). Cancellation will
    /// also cause the sampling to be stopped.
    pub async fn sample(&mut self, buf: &mut [i16; N]) {
        // Reset ADC fifo
        self.info.regs.ctrl().modify(|_, w| w.rstfifo().rstfifo_1());

        // Set fifo watermark
        self.info
            .regs
            .fctrl()
            .write(|w| unsafe { w.fwmark().bits((buf.len() - 1) as u8) });

        // Enable the watermark interrupt
        self.info.regs.ie().write(|w| w.fwmie().fwmie_1());

        // Send software trigger
        self.info.regs.swtrig().write(|w| w.swt0().swt0_1());

        // Wait for fifo watermark interrupt.
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            // Make sure there is at least one sample from each channel
            //   in the fifo
            if self.info.regs.fctrl().read().fcount().bits() >= buf.len() as u8 {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        for e in buf {
            *e = self.info.regs.resfifo().read().d().bits() as i16;
        }

        // Disable the watermark interrupt
        self.info.regs.ie().write(|w| w.fwmie().fwmie_0());
    }
}

trait SealedInstance {
    fn info() -> Info;
}

/// ADC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + 'static + Send {
    /// Interrupt for this ADC instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl Instance for peripherals::ADC0 {
    type Interrupt = crate::interrupt::typelevel::ADC0;
}

impl SealedInstance for peripherals::ADC0 {
    fn info() -> Info {
        // SAFETY: safe from single executor
        Info {
            regs: unsafe { crate::pac::Adc0::steal() },
        }
    }
}

/// Voltage Reference
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Reference {
    /// ADC positive reference voltage
    VRefP = 0,
    /// 1.8 V internal reference
    VddaAdc1v8 = 1,
    // according to the data sheet, 1.8 V internal reference again???
    // VDDA_ADC1V8 = 2,
}

impl From<Reference> for adc0::cfg::Refsel {
    fn from(reference: Reference) -> Self {
        match reference {
            Reference::VRefP => adc0::cfg::Refsel::Refsel0,
            Reference::VddaAdc1v8 => adc0::cfg::Refsel::Refsel1,
        }
    }
}

/// ADC channel side
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Side {
    A,
    B,
}

impl From<Side> for adc0::cmdl::Absel {
    fn from(side: Side) -> Self {
        match side {
            Side::A => adc0::cmdl::Absel::Absel0,
            Side::B => adc0::cmdl::Absel::Absel1,
        }
    }
}

/// ADC channel
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct AdcChannel {
    pub ch: adc0::cmdl::Adch,
    pub side: Side,
}

pub(crate) trait SealedInput {
    fn channel(&self) -> AdcChannel;
}

/// A dual purpose (digital/analog) input that can be used as analog input to ADC peripheral.
#[allow(private_bounds)]
pub trait Input: SealedInput + Into<AnyInput> + Peripheral<P = Self> + Sized + 'static {}

/// A type-erased ADC input.
///
/// This allows using several inputs in situations that might require
/// them to be the same type, like putting them in an array.
pub struct AnyInput {
    channel: AdcChannel,
    pin: AnyPin,
}

impl Peripheral for AnyInput {
    type P = AnyInput;

    unsafe fn clone_unchecked(&self) -> Self::P {
        Self {
            channel: self.channel,
            pin: self.pin.clone_unchecked(),
        }
    }
}

impl SealedInput for AnyInput {
    fn channel(&self) -> AdcChannel {
        self.channel
    }
}

impl Input for AnyInput {}

struct GuardedAnyInput<'a> {
    inner: PeripheralRef<'a, AnyInput>,
}

impl<'a> From<PeripheralRef<'a, AnyInput>> for GuardedAnyInput<'a> {
    fn from(val: PeripheralRef<'a, AnyInput>) -> Self {
        GuardedAnyInput { inner: val }
    }
}

impl<'a> Deref for GuardedAnyInput<'a> {
    type Target = PeripheralRef<'a, AnyInput>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for GuardedAnyInput<'_> {
    fn drop(&mut self) {
        self.inner.pin.reset();
    }
}

/// Macro to implement required types for dual purpose pins
macro_rules! impl_pin {
    ($pin:ident, $ch:ident, $side:ident) => {
        impl_pin!(@local, crate::peripherals::$pin, $ch, $side);
    };
    (@local, $pin:ty, $ch:ident, $side:ident) => {
        impl crate::adc::SealedInput for $pin {
            fn channel(&self) -> crate::adc::AdcChannel {
                self.set_function(Function::F0)
                    .set_pull(Pull::None)
                    .disable_input_buffer()
                    .set_slew_rate(SlewRate::Standard)
                    .set_drive_strength(DriveStrength::Normal)
                    .enable_analog_multiplex()
                    .set_drive_mode(DriveMode::PushPull)
                    .set_input_inverter(Inverter::Disabled);

                AdcChannel {
                    ch: crate::pac::adc0::cmdl::Adch::$ch,
                    side: crate::adc::Side::$side
                }
            }
        }

        impl crate::adc::Input for $pin {}

        impl From<$pin> for crate::adc::AnyInput {
            fn from(val: $pin) -> Self {
                crate::adc::AnyInput {
                    channel: val.channel(),
                    pin: val.into(),
                }
            }
        }
    };
}

impl_pin!(PIO0_5, Adch0, A);
impl_pin!(PIO0_6, Adch0, B);
impl_pin!(PIO0_12, Adch1, A);
impl_pin!(PIO0_13, Adch1, B);
impl_pin!(PIO0_19, Adch2, A);
impl_pin!(PIO0_20, Adch2, B);
impl_pin!(PIO0_26, Adch3, A);
impl_pin!(PIO0_27, Adch3, B);
impl_pin!(PIO1_8, Adch4, A);
impl_pin!(PIO1_9, Adch4, B);
impl_pin!(PIO3_23, Adch5, A);
impl_pin!(PIO3_24, Adch5, B);
