//! Implements I2C function support over flexcomm + gpios

#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};

use crate::iopctl::IopctlPin as Pin;

/// Bus speed (nominal SCL, no clock stretching)
pub enum Speed {
    /// 100 kbit/s
    Standard,

    /// 400 kbit/s
    Fast,

    /// 1 Mbit/s
    FastPlus,

    /// 3.4Mbit/s only available for slave devices
    High,
}

/// I2C address type
#[derive(Copy, Clone, Debug)]
pub struct Address(u8);

impl Address {
    /// Construct an address type
    pub const fn new(addr: u8) -> Option<Self> {
        match addr {
            0x08..=0x77 => Some(Self(addr)),
            _ => None,
        }
    }

    /// interpret address as a read command
    pub fn read(&self) -> u8 {
        (self.0 << 1) | 1
    }

    /// interpret address as a write command
    pub fn write(&self) -> u8 {
        self.0 << 1
    }
}

impl From<Address> for u8 {
    fn from(value: Address) -> Self {
        value.0
    }
}

/// specific information regarding transfer errors
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransferError {
    /// Timeout error
    Timeout,
    /// Reading from i2c failed
    ReadFail,
    /// Writing to i2c failed
    WriteFail,
    /// I2C Address not ACK'd
    AddressNack,
    /// Bus level arbitration loss
    ArbitrationLoss,
    /// Address + Start/Stop error
    StartStopError,
    /// state mismatch or other internal register unexpected state
    OtherBusError,
}

/// Error information type
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// propagating a lower level flexcomm error
    Flex(crate::flexcomm::Error),

    /// configuration requested is not supported
    UnsupportedConfiguration,

    /// transaction failure types
    Transfer(TransferError),
}

/// shorthand for -> Result<T>
pub type Result<T> = core::result::Result<T, Error>;

// implementing from allows ? operator from flexcomm::Result<T>
impl From<crate::flexcomm::Error> for Error {
    fn from(value: crate::flexcomm::Error) -> Self {
        Error::Flex(value)
    }
}

impl From<TransferError> for Error {
    fn from(value: TransferError) -> Self {
        Error::Transfer(value)
    }
}

mod sealed {
    /// simply seal a trait
    pub trait Sealed {}
}

impl<T: Pin> sealed::Sealed for T {}

/// shared functions between master and slave operation
#[allow(private_bounds)]
pub trait Instance: crate::flexcomm::I2cPeripheral {}
impl Instance for crate::peripherals::FLEXCOMM0 {}
impl Instance for crate::peripherals::FLEXCOMM1 {}
impl Instance for crate::peripherals::FLEXCOMM2 {}
impl Instance for crate::peripherals::FLEXCOMM3 {}
impl Instance for crate::peripherals::FLEXCOMM4 {}
impl Instance for crate::peripherals::FLEXCOMM5 {}
impl Instance for crate::peripherals::FLEXCOMM6 {}
impl Instance for crate::peripherals::FLEXCOMM7 {}

/// io configuration trait for easier configuration
pub trait SclPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for SCL usage
    fn as_scl(&self, pull: crate::iopctl::Pull);
}

/// io configuration trait for easier configuration
pub trait SdaPin<Instance>: Pin + sealed::Sealed + crate::Peripheral {
    /// convert the pin to appropriate function for SDA usage
    fn as_sda(&self, pull: crate::iopctl::Pull);
}

/// use FCn as I2C Master controller
pub struct I2cMaster<'a, FC: Instance> {
    bus: crate::flexcomm::I2cBus<'a, FC>,
    timeout: TimeoutSettings,
    #[cfg(feature = "time")]
    poll_start: Instant,
}

/// use FCn as I2C Slave controller
pub struct I2cSlave<'a, FC: Instance> {
    bus: crate::flexcomm::I2cBus<'a, FC>,
}

/// configuration struct for i2c master timeout control
pub struct TimeoutSettings {
    /// true - enable HW based timeout, false - disable
    pub hw_timeout: bool,

    /// software driven timeout duration, if time feature is enabled
    #[cfg(feature = "time")]
    pub sw_timeout: Duration,
}

impl<'a, FC: Instance> I2cMaster<'a, FC> {
    /// use flexcomm fc with Pins scl, sda as an I2C Master bus, configuring to speed and pull
    pub fn new(
        fc: impl Instance<P = FC> + 'a,
        scl: impl SclPin<FC> + 'a,
        sda: impl SdaPin<FC> + 'a,
        // TODO - integrate clock APIs to allow dynamic freq selection | clock: crate::flexcomm::Clock,
        pull: crate::iopctl::Pull,
        speed: Speed,
        timeout: TimeoutSettings,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;

        sda.as_sda(pull);
        scl.as_scl(pull);

        let bus: crate::flexcomm::I2cBus<'_, FC> = crate::flexcomm::I2cBus::new(fc, clock)?;

        // this check should be redundant with T::set_mode()? above

        // rates taken assuming SFRO:
        //
        //  7 => 403.3 kHz
        //  9 => 322.6 kHz
        // 12 => 247.8 kHz
        // 16 => 198.2 kHz
        // 18 => 166.6 Khz
        // 22 => 142.6 kHz
        // 30 => 100.0 kHz
        match speed {
            // 100 kHz
            Speed::Standard => bus.i2c().clkdiv().write(|w|
                // SAFETY: only unsafe due to .bits usage
                unsafe { w.divval().bits(30) }),

            // 400 kHz
            Speed::Fast => bus.i2c().clkdiv().write(|w|
                // SAFETY: only unsafe due to .bits usage
                unsafe { w.divval().bits(7) }),

            _ => return Err(Error::UnsupportedConfiguration),
        }

        bus.i2c().msttime().write(|w|
            // SAFETY: only unsafe due to .bits usage
            unsafe { w.mstsclhigh().bits(0).mstscllow().bits(1) });

        if timeout.hw_timeout {
            bus.i2c().timeout().write(|w|
                    // SAFETY: only unsafe due to .bits usage
                unsafe { w.to().bits(4096 >> 4) });

            bus.i2c().cfg().modify(|_, w| w.timeouten().enabled());
        }

        bus.i2c().intenset().write(|w|
                // SAFETY: only unsafe due to .bits usage
                unsafe { w.bits(0) });

        bus.i2c().cfg().write(|w| w.msten().set_bit());
        let mut this = Self {
            bus,
            timeout,
            #[cfg(feature = "time")]
            poll_start: Instant::now(),
        };
        this.poll_ready()?;

        Ok(this)
    }

    fn start(&mut self, address: u8, is_read: bool) -> Result<()> {
        let i2cregs = self.bus.i2c();

        // cannot start if not in IDLE state
        if !i2cregs.stat().read().mststate().is_idle() {
            return Err(TransferError::OtherBusError.into());
        }

        i2cregs.mstdat().write(|w|
            // SAFETY: only unsafe due to .bits usage
            unsafe { w.data().bits(address << 1 | if is_read {0x01} else {0x00}) });

        i2cregs.mstctl().write(|w| w.mststart().set_bit());

        self.poll_ready()?;

        if i2cregs.stat().read().mststate().is_nack_address() {
            // STOP bit to complete the attempted transfer
            self.stop()?;

            return Err(TransferError::AddressNack.into());
        }

        if is_read && !i2cregs.stat().read().mststate().is_receive_ready() {
            return Err(TransferError::ReadFail.into());
        }

        if !is_read && !i2cregs.stat().read().mststate().is_transmit_ready() {
            return Err(TransferError::WriteFail.into());
        }

        self.check_for_bus_errors()
    }

    fn check_for_bus_errors(&self) -> Result<()> {
        let i2cregs = self.bus.i2c();

        if i2cregs.stat().read().mstarbloss().is_arbitration_loss() {
            Err(TransferError::ArbitrationLoss.into())
        } else if i2cregs.stat().read().mstststperr().is_error() {
            Err(TransferError::StartStopError.into())
        } else {
            Ok(())
        }
    }

    fn read_no_stop(&mut self, address: u8, read: &mut [u8]) -> Result<()> {
        let i2cregs = self.bus.i2c();

        self.start(address, true)?;

        let read_len = read.len();

        for (i, r) in read.iter_mut().enumerate() {
            self.poll_ready()?;

            // check transmission continuity
            if !i2cregs.stat().read().mststate().is_receive_ready() {
                return Err(TransferError::ReadFail.into());
            }

            self.check_for_bus_errors()?;

            *r = i2cregs.mstdat().read().data().bits();

            // continue after ACK until last byte
            if i < read_len - 1 {
                i2cregs.mstctl().write(|w| w.mstcontinue().set_bit());
            }
        }

        Ok(())
    }

    fn write_no_stop(&mut self, address: u8, write: &[u8]) -> Result<()> {
        // Procedure from 24.3.1.1 pg 545
        let i2cregs = self.bus.i2c();

        self.start(address, false)?;

        for byte in write.iter() {
            i2cregs.mstdat().write(|w|
                // SAFETY: unsafe only due to .bits usage
                unsafe { w.data().bits(*byte) });

            i2cregs.mstctl().write(|w| w.mstcontinue().set_bit());

            self.poll_ready()?;
            self.check_for_bus_errors()?;
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        // Procedure from 24.3.1.1 pg 545
        let i2cregs = self.bus.i2c();

        i2cregs.mstctl().write(|w| w.mststop().set_bit());
        self.poll_ready()?;
        self.check_for_bus_errors()?;

        // ensure return to idle state for bus (no stuck SCL/SDA lines)
        if i2cregs.stat().read().mststate().is_idle() {
            Ok(())
        } else {
            Err(TransferError::OtherBusError.into())
        }
    }

    fn check_timeout(&mut self) -> Result<()> {
        let stat = self.bus.i2c().stat().read();
        if self.timeout.hw_timeout && (stat.scltimeout().bit_is_set() || stat.eventtimeout().is_even_timeout()) {
            Err(TransferError::Timeout.into())
        } else {
            #[cfg(feature = "time")]
            {
                if Instant::now() - self.poll_start >= self.timeout.sw_timeout {
                    return Err(TransferError::Timeout.into());
                }
            }

            Ok(())
        }
    }

    fn poll_ready(&mut self) -> Result<()> {
        #[cfg(feature = "time")]
        {
            self.poll_start = Instant::now();
        }

        while self.bus.i2c().stat().read().mstpending().bit_is_clear() {
            self.check_timeout()?;
        }

        Ok(())
    }
}

// re-export embedded-hal I2c trait
pub use embedded_hal_1::i2c::{ErrorType as I2cMasterBlockingErrorType, I2c as I2cMasterBlocking};

/// Error Types for I2C communication
impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Flex(_) => embedded_hal_1::i2c::ErrorKind::Bus,
            Self::UnsupportedConfiguration => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Transfer(e) => match e {
                TransferError::Timeout => embedded_hal_1::i2c::ErrorKind::Other,
                TransferError::ReadFail | TransferError::WriteFail => {
                    embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Data)
                }
                TransferError::AddressNack => {
                    embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
                }
                TransferError::ArbitrationLoss => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
                TransferError::StartStopError => embedded_hal_1::i2c::ErrorKind::Bus,
                TransferError::OtherBusError => embedded_hal_1::i2c::ErrorKind::Bus,
            },
        }
    }
}

impl<'a, FC: Instance> I2cMasterBlockingErrorType for I2cMaster<'a, FC> {
    type Error = Error;
}

// implement generic i2c interface for peripheral master type
impl<'a, FC: Instance> I2cMasterBlocking for I2cMaster<'a, FC> {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<()> {
        self.read_no_stop(address, read)?;
        self.stop()
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<()> {
        self.write_no_stop(address, write)?;
        self.stop()
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<()> {
        self.write_no_stop(address, write)?;
        self.read_no_stop(address, read)?;
        self.stop()
    }

    fn transaction(&mut self, address: u8, operations: &mut [embedded_hal_1::i2c::Operation<'_>]) -> Result<()> {
        let needs_stop = !operations.is_empty();

        for op in operations {
            match op {
                embedded_hal_1::i2c::Operation::Read(read) => {
                    self.read_no_stop(address, read)?;
                }
                embedded_hal_1::i2c::Operation::Write(write) => {
                    self.write_no_stop(address, write)?;
                }
            }
        }

        if needs_stop {
            self.stop()?;
        }

        Ok(())
    }
}

/// interface trait for generalized I2C slave interactions
pub trait I2cSlaveBlocking {
    /// block until the address is pinged (expect no payload)
    fn block_until_addressed(&self) -> Result<()>;

    /// wait for a read request
    fn read(&self, read: &mut [u8]) -> Result<()>;

    /// wait for a write request
    fn write(&self, write: &[u8]) -> Result<()>;
}

impl<'a, FC: Instance> I2cSlave<'a, FC> {
    /// use flexcomm fc with Pins scl, sda as an I2C Master bus, configuring to speed and pull
    pub fn new(
        fc: impl Instance<P = FC> + 'a,
        scl: impl SclPin<FC>,
        sda: impl SdaPin<FC>,
        // TODO - integrate clock APIs to allow dynamic freq selection | clock: crate::flexcomm::Clock,
        pull: crate::iopctl::Pull,
        address: Address,
    ) -> Result<Self> {
        // TODO - clock integration
        let clock = crate::flexcomm::Clock::Sfro;

        sda.as_sda(pull);
        scl.as_scl(pull);

        let bus = crate::flexcomm::I2cBus::new(fc, clock)?;

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

        Ok(Self { bus })
    }

    fn poll(&self) -> Result<()> {
        let i2c = self.bus.i2c();

        while i2c.stat().read().slvpending().is_in_progress() {}

        Ok(())
    }
}

impl<'a, FC: Instance> I2cSlaveBlocking for I2cSlave<'a, FC> {
    fn block_until_addressed(&self) -> Result<()> {
        self.poll()?;

        let i2c = self.bus.i2c();

        if !i2c.stat().read().slvstate().is_slave_address() {
            return Err(TransferError::AddressNack.into());
        }

        i2c.slvctl().write(|w| w.slvcontinue().continue_());
        Ok(())
    }

    fn read(&self, read: &mut [u8]) -> Result<()> {
        let i2c = self.bus.i2c();

        self.block_until_addressed()?;

        for b in read {
            self.poll()?;

            if !i2c.stat().read().slvstate().is_slave_receive() {
                return Err(TransferError::ReadFail.into());
            }

            *b = i2c.slvdat().read().data().bits();

            i2c.slvctl().write(|w| w.slvcontinue().continue_());
        }

        Ok(())
    }

    fn write(&self, write: &[u8]) -> Result<()> {
        let i2c = self.bus.i2c();

        self.block_until_addressed()?;

        for b in write {
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

// flexcomm <-> Pin function map
macro_rules! impl_scl {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl SclPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_scl(&self, pull: crate::iopctl::Pull) {
                // UM11147 table 299 pg 262+
                self.set_pull(pull)
                    .set_slew_rate(crate::gpio::SlewRate::Standard)
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
                    .set_drive_mode(crate::gpio::DriveMode::OpenDrain)
                    .set_input_polarity(crate::gpio::Polarity::ActiveHigh)
                    .enable_input_buffer()
                    .set_function(crate::iopctl::Function::$fn);
            }
        }
    };
}
macro_rules! impl_sda {
    ($piom_n:ident, $fn:ident, $fcn:ident) => {
        impl SdaPin<crate::peripherals::$fcn> for crate::peripherals::$piom_n {
            fn as_sda(&self, pull: crate::iopctl::Pull) {
                // UM11147 table 299 pg 262+
                self.set_pull(pull)
                    .set_slew_rate(crate::gpio::SlewRate::Standard)
                    .set_drive_strength(crate::gpio::DriveStrength::Normal)
                    .set_drive_mode(crate::gpio::DriveMode::OpenDrain)
                    .set_input_polarity(crate::gpio::Polarity::ActiveHigh)
                    .enable_input_buffer()
                    .set_function(crate::iopctl::Function::$fn);
            }
        }
    };
}

// Flexcomm0 GPIOs -
impl_scl!(PIO0_1, F1, FLEXCOMM0);
impl_sda!(PIO0_2, F1, FLEXCOMM0);

impl_scl!(PIO3_1, F5, FLEXCOMM0);
impl_sda!(PIO3_2, F5, FLEXCOMM0);
impl_sda!(PIO3_3, F5, FLEXCOMM0);
impl_scl!(PIO3_4, F5, FLEXCOMM0);

// Flexcomm1 GPIOs -
impl_scl!(PIO0_8, F1, FLEXCOMM1);
impl_sda!(PIO0_9, F1, FLEXCOMM1);
impl_sda!(PIO0_10, F1, FLEXCOMM1);
impl_scl!(PIO0_11, F1, FLEXCOMM1);

impl_scl!(PIO7_26, F1, FLEXCOMM1);
impl_sda!(PIO7_27, F1, FLEXCOMM1);
impl_sda!(PIO7_28, F1, FLEXCOMM1);
impl_scl!(PIO7_29, F1, FLEXCOMM1);

// Flexcomm2 GPIOs -
impl_scl!(PIO0_15, F1, FLEXCOMM2);
impl_sda!(PIO0_16, F1, FLEXCOMM2);
impl_sda!(PIO0_17, F1, FLEXCOMM2);
impl_scl!(PIO0_18, F1, FLEXCOMM2);

impl_sda!(PIO4_8, F5, FLEXCOMM2);

impl_scl!(PIO7_30, F5, FLEXCOMM2);
impl_sda!(PIO7_31, F5, FLEXCOMM2);

// Flexcomm3 GPIOs -
impl_scl!(PIO0_22, F1, FLEXCOMM3);
impl_sda!(PIO0_23, F1, FLEXCOMM3);
impl_sda!(PIO0_24, F1, FLEXCOMM3);
impl_scl!(PIO0_25, F1, FLEXCOMM3);

// Flexcomm4 GPIOs -
impl_scl!(PIO0_29, F1, FLEXCOMM4);
impl_sda!(PIO0_30, F1, FLEXCOMM4);
impl_sda!(PIO0_31, F1, FLEXCOMM4);
impl_scl!(PIO1_0, F1, FLEXCOMM4);

// Flexcomm5 GPIOs -
impl_scl!(PIO1_4, F1, FLEXCOMM5);
impl_sda!(PIO1_5, F1, FLEXCOMM5);
impl_sda!(PIO1_6, F1, FLEXCOMM5);
impl_scl!(PIO1_7, F1, FLEXCOMM5);

impl_scl!(PIO3_16, F4, FLEXCOMM5);
impl_sda!(PIO3_17, F4, FLEXCOMM5);
impl_sda!(PIO3_18, F4, FLEXCOMM5);
impl_scl!(PIO3_22, F5, FLEXCOMM5);

// Flexcomm6 GPIOs -
impl_scl!(PIO3_26, F1, FLEXCOMM6);
impl_sda!(PIO3_27, F1, FLEXCOMM6);
impl_sda!(PIO3_28, F1, FLEXCOMM6);
impl_scl!(PIO3_29, F1, FLEXCOMM6);

// Flexcomm7 GPIOs -
impl_scl!(PIO4_1, F1, FLEXCOMM7);
impl_sda!(PIO4_2, F1, FLEXCOMM7);
impl_sda!(PIO4_3, F1, FLEXCOMM7);
impl_scl!(PIO4_4, F1, FLEXCOMM7);
