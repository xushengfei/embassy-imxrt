//! Implements I2C function support over flexcomm + gpios

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use super::{Async, Blocking, Instance, Mode, Result, SclPin, SdaPin, TransferError};
use crate::{dma, Peripheral};

/// I2C address type
#[derive(Copy, Clone, Debug)]
pub struct Address(u8);

impl Address {
    /// Construct an address type
    #[must_use]
    pub const fn new(addr: u8) -> Option<Self> {
        match addr {
            0x08..=0x77 => Some(Self(addr)),
            _ => None,
        }
    }

    /// interpret address as a read command
    #[must_use]
    pub fn read(&self) -> u8 {
        (self.0 << 1) | 1
    }

    /// interpret address as a write command
    #[must_use]
    pub fn write(&self) -> u8 {
        self.0 << 1
    }
}

impl From<Address> for u8 {
    fn from(value: Address) -> Self {
        value.0
    }
}

/// use `FCn` as I2C Slave controller
pub struct I2cSlave<'a, FC: Instance, M: Mode, D: dma::Instance> {
    bus: crate::flexcomm::I2cBus<'a, FC>,
    _phantom: PhantomData<M>,
    dma_ch: Option<dma::channel::ChannelAndRequest<'a, D>>,
}

/// interface trait for generalized I2C slave interactions
pub trait I2cSlaveBlocking {
    /// listen for cmd
    fn listen(&self, cmd: &mut [u8]) -> Result<()>;

    /// respond with data
    fn respond(&self, response: &[u8]) -> Result<()>;
}

/// interface trait for generalized I2C slave interactions
pub trait I2cSlaveAsync {
    /// listen for cmd
    async fn listen(&mut self, cmd: &mut [u8], expect_stop: bool) -> Result<()>;

    /// respond with data
    async fn respond(&mut self, response: &[u8]) -> Result<()>;
}

impl<'a, FC: Instance, M: Mode, D: dma::Instance> I2cSlave<'a, FC, M, D> {
    /// use flexcomm fc with Pins scl, sda as an I2C Master bus, configuring to speed and pull
    fn new_inner(
        bus: crate::flexcomm::I2cBus<'a, FC>,
        scl: impl SclPin<FC>,
        sda: impl SdaPin<FC>,
        // TODO - integrate clock APIs to allow dynamic freq selection | clock: crate::flexcomm::Clock,
        pull: crate::iopctl::Pull,
        address: Address,
        dma_ch: Option<dma::channel::ChannelAndRequest<'a, D>>,
    ) -> Result<Self> {
        sda.as_sda(pull);
        scl.as_scl(pull);

        // this check should be redundant with T::set_mode()? above
        let i2c = bus.i2c();

        // rates taken assuming SFRO:
        //
        //  7 => 403.3 kHz
        //  9 => 322.6 kHz
        // 12 => 247.8 kHz
        // 16 => 198.2 kHz
        // 18 => 166.6 Khz
        // 22 => 142.6 kHz
        // 30 => 100.0 kHz
        // UM10204 pg.44 rev7
        // tSU;DAT >= 250ns -> < 250MHz
        i2c.clkdiv().write(|w|
            // SAFETY: only unsafe due to .bits usage
            unsafe { w.divval().bits(0) });

        // address 0 match = addr, per UM11147 24.3.2.1
        i2c.slvadr(0).modify(|_, w|
            // note: shift is omitted as performed via w.slvadr() 
            // SAFETY: unsafe only required due to use of unnamed "bits" field
            unsafe {w.slvadr().bits(address.0)}.sadisable().enabled());

        // SLVEN = 1, per UM11147 24.3.2.1
        i2c.cfg().write(|w| w.slven().enabled());

        Ok(Self {
            bus,
            _phantom: PhantomData,
            dma_ch,
        })
    }
}

impl<'a, FC: Instance, D: dma::Instance> I2cSlave<'a, FC, Blocking, D> {
    /// use flexcomm fc with Pins scl, sda as an I2C Master bus, configuring to speed and pull
    pub fn new_blocking(
        fc: impl Instance<P = FC> + 'a,
        scl: impl SclPin<FC>,
        sda: impl SdaPin<FC>,
        // TODO - integrate clock APIs to allow dynamic freq selection | clock: crate::flexcomm::Clock,
        pull: crate::iopctl::Pull,
        address: Address,
        _dma_ch: impl Peripheral<P = D> + 'a,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;
        let bus = crate::flexcomm::I2cBus::new_blocking(fc, clock)?;

        Self::new_inner(bus, scl, sda, pull, address, None)
    }

    fn poll(&self) -> Result<()> {
        let i2c = self.bus.i2c();

        while i2c.stat().read().slvpending().is_in_progress() {}

        Ok(())
    }

    fn block_until_addressed(&self) -> Result<()> {
        self.poll()?;

        let i2c = self.bus.i2c();

        if !i2c.stat().read().slvstate().is_slave_address() {
            return Err(TransferError::AddressNack.into());
        }

        i2c.slvctl().write(|w| w.slvcontinue().continue_());
        Ok(())
    }
}

impl<'a, FC: Instance, D: dma::Instance> I2cSlave<'a, FC, Async, D> {
    /// use flexcomm fc with Pins scl, sda as an I2C Master bus, configuring to speed and pull
    pub fn new_async(
        fc: impl Instance<P = FC> + 'a,
        scl: impl SclPin<FC>,
        sda: impl SdaPin<FC>,
        // TODO - integrate clock APIs to allow dynamic freq selection | clock: crate::flexcomm::Clock,
        pull: crate::iopctl::Pull,
        address: Address,
        dma_ch: impl Peripheral<P = D> + 'a,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;
        let bus = crate::flexcomm::I2cBus::new_async(fc, clock)?;
        let ch = dma::Dma::reserve_channel(dma_ch);
        Self::new_inner(bus, scl, sda, pull, address, Some(ch))
    }

    async fn block_until_addressed(&self) -> Result<()> {
        let i2c = self.bus.i2c();

        i2c.intenset()
            .write(|w| w.slvpendingen().set_bit().slvdeselen().set_bit());

        poll_fn(|cx: &mut core::task::Context<'_>| {
            self.bus.waker().register(cx.waker());
            if i2c.stat().read().slvpending().bit_is_set() {
                return Poll::Ready(());
            }

            if i2c.stat().read().slvdesel().bit_is_set() {
                i2c.stat().write(|w| w.slvdesel().deselected());
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        i2c.intenclr()
            .write(|w| w.slvpendingclr().set_bit().slvdeselclr().set_bit());

        if !i2c.stat().read().slvstate().is_slave_address() {
            return Err(TransferError::AddressNack.into());
        }

        i2c.slvctl().modify(|_, w| w.slvcontinue().continue_());

        Ok(())
    }
}

impl<FC: Instance, D: dma::Instance> I2cSlaveBlocking for I2cSlave<'_, FC, Blocking, D> {
    fn listen(&self, cmd: &mut [u8]) -> Result<()> {
        let i2c = self.bus.i2c();

        // Skip address phase if we are already in receive mode
        if !i2c.stat().read().slvstate().is_slave_receive() {
            self.block_until_addressed()?;
        }

        for b in cmd {
            self.poll()?;

            if !i2c.stat().read().slvstate().is_slave_receive() {
                return Err(TransferError::ReadFail.into());
            }

            *b = i2c.slvdat().read().data().bits();

            i2c.slvctl().write(|w| w.slvcontinue().continue_());
        }

        Ok(())
    }

    fn respond(&self, response: &[u8]) -> Result<()> {
        let i2c = self.bus.i2c();

        self.block_until_addressed()?;

        for b in response {
            self.poll()?;

            if !i2c.stat().read().slvstate().is_slave_transmit() {
                return Err(TransferError::WriteFail.into());
            }

            i2c.slvdat().write(|w|
                    // SAFETY: unsafe only here due to use of bits()
                    unsafe{w.data().bits(*b)});

            i2c.slvctl().write(|w| w.slvcontinue().continue_());
        }

        Ok(())
    }
}

impl<FC: Instance, D: dma::Instance> I2cSlaveAsync for I2cSlave<'_, FC, Async, D> {
    async fn listen(&mut self, request: &mut [u8], expect_stop: bool) -> Result<()> {
        let i2c = self.bus.i2c();

        // Skip address phase if we are already in receive mode
        if !i2c.stat().read().slvstate().is_slave_receive() {
            self.block_until_addressed().await?;
        }

        // Verify that we are ready to receive after addressed
        if !i2c.stat().read().slvstate().is_slave_receive() {
            return Err(TransferError::ReadFail.into());
        }

        // Enable DMA
        i2c.slvctl().write(|w| w.slvdma().enabled());

        // Enable interrupt
        i2c.intenset()
            .write(|w| w.slvpendingen().set_bit().slvdeselen().set_bit());

        let options = dma::transfer::TransferOptions::default();
        self.dma_ch
            .as_mut()
            .unwrap()
            .read_from_peripheral(i2c.slvdat().as_ptr() as *mut u8, request, options);

        poll_fn(|cx| {
            let i2c = self.bus.i2c();
            self.bus.waker().register(cx.waker());
            self.dma_ch.as_ref().unwrap().get_waker().register(cx.waker());

            //check for readyness
            if i2c.stat().read().slvpending().bit_is_set() {
                return Poll::Ready(());
            }

            if i2c.stat().read().slvdesel().bit_is_set() {
                i2c.stat().write(|w| w.slvdesel().deselected());
                return Poll::Ready(());
            }

            // Only check DMA status if we are not expecting a stop
            if !expect_stop && !self.dma_ch.as_ref().unwrap().is_active() {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        // Disable interrupts
        i2c.intenclr()
            .write(|w| w.slvpendingclr().set_bit().slvdeselclr().set_bit());

        Ok(())
    }

    async fn respond(&mut self, response: &[u8]) -> Result<()> {
        let i2c = self.bus.i2c();
        self.block_until_addressed().await?;

        // Verify that we are ready for transmit after addressed
        if !i2c.stat().read().slvstate().is_slave_transmit() {
            return Err(TransferError::WriteFail.into());
        }

        // Enable DMA
        i2c.slvctl().write(|w| w.slvdma().enabled());

        // Enable interrupt
        i2c.intenset()
            .write(|w| w.slvpendingen().set_bit().slvdeselen().set_bit());

        let options = dma::transfer::TransferOptions::default();
        self.dma_ch
            .as_mut()
            .unwrap()
            .write_to_peripheral(response, i2c.slvdat().as_ptr() as *mut u8, options);

        poll_fn(|cx| {
            let i2c = self.bus.i2c();
            self.bus.waker().register(cx.waker());
            self.dma_ch.as_ref().unwrap().get_waker().register(cx.waker());

            if i2c.stat().read().slvpending().bit_is_set() {
                return Poll::Ready(());
            }

            if i2c.stat().read().slvdesel().bit_is_set() {
                i2c.stat().write(|w| w.slvdesel().deselected());
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        // Disable interrupts
        i2c.intenclr()
            .write(|w| w.slvpendingclr().set_bit().slvdeselclr().set_bit());

        Ok(())
    }
}
