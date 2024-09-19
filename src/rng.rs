//! True Random Number Generator (TRNG)

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_futures::block_on;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use rand_core::{CryptoRng, RngCore};

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, peripherals, Peripheral};

static RNG_WAKER: AtomicWaker = AtomicWaker::new();

/// RNG ;error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Seed error.
    SeedError,
}

/// RNG interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().int_status().read().ent_val().bit_is_set() {
            T::regs().int_ctrl().modify(|_, w| w.ent_val().clear_bit());
            RNG_WAKER.wake();
        }
    }
}

/// RNG driver.
pub struct Rng<'d, T: Instance> {
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Rng<'d, T> {
    /// Create a new RNG driver.
    pub fn new(
        inner: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        // SAFETY: safe from single executor
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
        // SAFETY: safe from single executor
        let rstctl0 = unsafe { crate::pac::Rstctl0::steal() };

        clkctl0.pscctl0_set().write(|w| w.rng_clk().set_clock());
        rstctl0.prstctl0_clr().write(|w| w.rng().clr_reset());

        into_ref!(inner);

        let mut random = Self { _inner: inner };
        random.reset();

        // Mask all interrupts
        T::regs().int_mask().write(|w| {
            w.ent_val()
                .ent_val_0()
                .hw_err()
                .hw_err_0()
                .frq_ct_fail()
                .frq_ct_fail_0()
        });

        // Switch TRNG to programming mode
        T::regs().mctl().modify(|_, w| w.prgm().set_bit());

        // Enable ENT_VAL interrupt
        T::regs().int_ctrl().write(|w| w.ent_val().ent_val_1());
        T::regs().int_mask().write(|w| w.ent_val().ent_val_1());

        // Switch TRNG to Run Mode
        T::regs()
            .mctl()
            .modify(|_, w| w.trng_acc().set_bit().prgm().clear_bit());

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        random
    }

    /// Reset the RNG.
    pub fn reset(&mut self) {
        T::regs().mctl().write(|w| w.rst_def().set_bit().prgm().set_bit());
    }

    /// Fill the given slice with random values.
    pub async fn async_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        // We have a total of 16 words (512 bits) of entropy at our
        // disposal. The idea here is to read all bits and copy the
        // necessary bytes to the slice.
        for chunk in dest.chunks_mut(64) {
            let mut bits = T::regs().mctl().read();

            if bits.ent_val().bit_is_clear() {
                // wait for interrupt
                poll_fn(|cx| {
                    // Check if already ready.
                    if T::regs().int_status().read().ent_val().bit_is_set() {
                        return Poll::Ready(());
                    }

                    RNG_WAKER.register(cx.waker());

                    T::regs().int_mask().modify(|_, w| w.ent_val().ent_val_1());

                    // Check again if interrupt fired
                    if T::regs().mctl().read().ent_val().bit_is_set() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;

                bits = T::regs().mctl().read();
            }

            if bits.ent_val().bit_is_set() {
                let mut entropy = [0; 16];

                for (i, item) in entropy.iter_mut().enumerate() {
                    *item = T::regs().ent(i).read().bits();
                }

                // Read MCTL after reading ENT15
                let _ = T::regs().mctl().read();

                if entropy.iter().any(|e| *e == 0) {
                    return Err(Error::SeedError);
                }

                // SAFETY: entropy is the same for input and output types in
                // native endianness.
                let entropy: [u8; 64] = unsafe { core::mem::transmute(entropy) };

                // write bytes to chunk
                for (dest, src) in chunk.iter_mut().zip(entropy.iter()) {
                    *dest = *src
                }
            }
        }

        Ok(())
    }
}

impl<'d, T: Instance> RngCore for Rng<'d, T> {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        block_on(self.async_fill_bytes(&mut bytes)).unwrap();
        u32::from_ne_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        block_on(self.async_fill_bytes(&mut bytes)).unwrap();
        u64::from_ne_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        block_on(self.async_fill_bytes(dest)).unwrap();
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl<'d, T: Instance> CryptoRng for Rng<'d, T> {}

trait SealedInstance {
    fn regs() -> crate::pac::Trng;
}

/// RNG instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + 'static + Send {
    /// Interrupt for this RNG instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl Instance for peripherals::RNG {
    type Interrupt = crate::interrupt::typelevel::RNG;
}

impl SealedInstance for peripherals::RNG {
    fn regs() -> crate::pac::Trng {
        // SAFETY: safe from single executor
        unsafe { crate::pac::Trng::steal() }
    }
}
