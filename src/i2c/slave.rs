//! Implements I2C Bus Slave Mode Support

use super::*;

/// I2C Slave mode controller over flexcomm T
#[allow(private_bounds)] // allow here for Sealed trait pattern
pub struct Slave<'d, T: I2cSlaveController> {
    addr: Address,
    _p: PeripheralRef<'d, T>,
}

/// error type for I2C Slave operations
#[derive(Copy, Clone, Debug)]
pub enum Error {
    /// an error occurred due to unexpected bus state
    SlaveState,

    /// address received was not for this device
    WrongAddress,
}

/// trait to generalize interaction over functions and methods
pub trait SlaveBlocking {
    /// block (spin loop) until N bytes or read or a bus error is encountered
    fn blocking_read<const N: usize>(&self) -> Result<[u8; N], Error>;

    /// block (spin loop) until out bytes are written or a bus error is encountered
    fn blocking_write(&self, out: &[u8]) -> Result<(), Error>;

    /// block until a ping (W or R to address) is received
    fn wait_for_ping(&self) -> Result<(), Error>;
}

#[allow(private_bounds)]
trait I2cSlaveController: I2CPeripheral {
    fn init_slave(addr: &Address) -> Result<(), Error>;

    fn poll();
    fn read_addr() -> Result<(), Error>;
    fn read_one() -> Result<u8, Error>;
    fn write_one(byte: u8) -> Result<(), Error>;
}

impl<T: I2CPeripheral> I2cSlaveController for T {
    fn init_slave(addr: &Address) -> Result<(), Error> {
        // first enable FC: UM11147 21.4
        //let fc = T::fc();
        // enable clock
        // set source + dividers
        // clear from reset
        // set interface to I2C
        T::enable_flexcomm();

        let reg = T::i2c();

        // SLVEN = 1, per UM11147 24.3.2.1
        reg.cfg()
            .modify(|_, w| w.slven().set_bit().monen().enabled().monclkstr().enabled());

        // address 0 match = addr, per UM11147 24.3.2.1
        reg.slvadr(0).modify(|_, w|
            // note: shift is omitted as performed via w.slvadr() 
            // SAFETY: unsafe only required due to use of unnamed "bits" field
            unsafe {w.slvadr().bits(addr.0)}.autonack().automatic().sadisable().enabled());

        Ok(())
    }

    fn poll() {
        let reg = T::i2c();
        // while !(state & SLVPENDING) {}, per UM11147 24.3.2.1
        while !reg.stat().read().slvpending().is_pending() {}
    }

    fn read_addr() -> Result<(), Error> {
        let reg = T::i2c();

        // wait for data from bus
        T::poll();

        // if stat & SLVSTATE != SLVST_ADDR -> Err, per UM11147 24.3.2.1
        if !reg.stat().read().slvstate().is_slave_address() {
            return Err(Error::WrongAddress);
        }

        // ACK address
        reg.slvctl().write(|w| w.slvcontinue().continue_());

        Ok(())
    }

    fn read_one() -> Result<u8, Error> {
        let reg = T::i2c();

        // wait for next element
        T::poll();

        if !reg.stat().read().slvstate().is_slave_receive() {
            return Err(Error::SlaveState);
        }

        let data = reg.slvdat().read().data().bits();

        // ACK the data
        reg.slvctl().write(|w| w.slvcontinue().continue_());

        Ok(data)
    }

    fn write_one(byte: u8) -> Result<(), Error> {
        let reg = T::i2c();

        T::poll();

        if !reg.stat().read().slvstate().is_slave_transmit() {
            return Err(Error::SlaveState);
        }

        reg.slvdat().write(|w|
            // SAFETY: only unsafe due to use of bits()
            unsafe {w.data().bits(byte)});

        reg.slvctl().write(|w| w.slvcontinue().continue_());

        Ok(())
    }
}

#[allow(private_bounds)]
impl<'d, T: I2cSlaveController<P = T>> Slave<'d, T> {
    /// construct an I2C Slave controller from T (flexcomm instance)
    pub fn new(consumed: T, addr: Address) -> Result<Self, Error> {
        T::init_slave(&addr)?;

        Ok(Self {
            addr,
            _p: consumed.into_ref(),
        })
    }

    /// fetch actively configured slave address
    pub fn address(&self) -> Address {
        self.addr
    }
}

#[allow(private_bounds)]
impl<'d, T: I2cSlaveController<P = T>> SlaveBlocking for Slave<'d, T> {
    /// perform a blocking (spin loop) I2C read for N bytes
    fn blocking_read<const N: usize>(&self) -> Result<[u8; N], Error> {
        let mut bytes = [0u8; N];

        T::read_addr()?;

        for b in &mut bytes {
            *b = T::read_one()?;
        }

        Ok(bytes)
    }

    /// perform a blocking (spin loop) I2C write
    fn blocking_write(&self, out: &[u8]) -> Result<(), Error> {
        T::read_addr()?;

        for o in out {
            T::write_one(*o)?;
        }

        Ok(())
    }

    fn wait_for_ping(&self) -> Result<(), Error> {
        T::read_addr()
    }
}
