//! Enhanced Serial Peripheral Interface (eSPI) driver.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;
use embassy_sync::waitqueue::AtomicWaker;
use paste::paste;

use crate::clocks::{enable_and_reset, SysconPeripheral};
use crate::gpio::{DriveMode, DriveStrength, Function, GpioPin as Pin, Inverter, Pull, SlewRate};
use crate::interrupt::typelevel::Interrupt;
pub use crate::pac::espi::espicap::{Flashmx, Maxspd, Safera, Spicap};
pub use crate::pac::espi::port::cfg::Direction;
use crate::pac::espi::port::cfg::Type;
pub use crate::pac::espi::port::ramuse::Len;
pub use crate::pac::espi::stataddr::Base;
use crate::{interrupt, peripherals, Peripheral};

// This controller has 5 different eSPI ports
const ESPI_PORTS: usize = 5;

static ESPI_WAKER: AtomicWaker = AtomicWaker::new();

/// Result type alias
pub type Result<T> = core::result::Result<T, Error>;

/// eSPI error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// CRC Error
    Crc,

    /// HStall Error
    HStall,
}

/// eSPI Command Length
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Length {
    /// 1 byte
    OneByte,

    /// 2 bytes
    TwoBytes,

    /// 4 bytes
    FourBytes,
}

impl From<Length> for u8 {
    fn from(length: Length) -> u8 {
        match length {
            Length::OneByte => 0,
            Length::TwoBytes => 1,
            Length::FourBytes => 3,
        }
    }
}

/// eSPI interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let stat = T::info().regs.intstat().read();
        T::info().regs.intenclr().write(|w| unsafe { w.bits(stat.bits()) });

        if stat.bus_rst().bit_is_set()
            || stat.port_int0().bit_is_set()
            || stat.port_int1().bit_is_set()
            || stat.port_int2().bit_is_set()
            || stat.port_int3().bit_is_set()
            || stat.port_int4().bit_is_set()
            || stat.p80int().bit_is_set()
            || stat.bus_rst().bit_is_set()
            || stat.irq_upd().bit_is_set()
            || stat.wire_chg().bit_is_set()
            || stat.hstall().bit_is_set()
            || stat.crcerr().bit_is_set()
            || stat.gpio().bit_is_set()
        {
            ESPI_WAKER.wake();
        }
    }
}

/// eSPI Port configuration.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PortConfig {
    /// Unconfigured
    Unconfigured,

    /// ACPI-style Endpoint
    AcpiEndpoint {
        /// Port Direction
        direction: Direction,

        /// Offset from 0 or the selected mapped base for matching
        /// memory or IO
        addr: u16,
    },

    /// ACPI-style Index/Data
    AcpiIndex,

    /// Mailbox Shared
    MailboxShared {
        /// Port Direction
        direction: Direction,

        /// Port address to the host
        addr: u16,

        /// Offset into RAM space
        offset: u16,

        /// Length of the mailbox or mastering area per direction.
        length: Len,
    },

    /// Mailbox Single
    MailboxSingle {
        /// Port Direction
        direction: Direction,

        /// Offset from 0 or the selected mapped base for matching
        /// memory or IO
        addr: u16,

        /// Word-aligned offset into the RAM
        offset: u16,

        /// This is the length of the mailbox or mastering area per
        /// direction.
        length: Len,
    },

    /// Mailbox Split
    MailboxSplit {
        /// Port Direction
        direction: Direction,

        /// Offset from 0 or the selected mapped base for matching
        /// memory or IO
        addr: u16,

        /// Word-aligned offset into the RAM
        offset: u16,

        /// This is the length of the mailbox or mastering area per
        /// direction.
        length: Len,
    },

    /// Put Posted/Completion Mem32
    PutPcMem32,

    /// Mailbox Split OOB
    MailboxSplitOOB,

    /// Slave Flash
    SlaveFlash,

    /// Mem Single
    MemSingle,

    /// Master Flash
    MasterFlash,
}

impl Into<Type> for PortConfig {
    fn into(self) -> Type {
        match self {
            PortConfig::Unconfigured => Type::Unconfigured,
            PortConfig::AcpiEndpoint { .. } => Type::AcpiEnd,
            PortConfig::AcpiIndex => Type::AcpiIndex,
            PortConfig::MailboxShared { .. } => Type::MailboxShared,
            PortConfig::MailboxSingle { .. } => Type::MailboxSingle,
            PortConfig::MailboxSplit { .. } => Type::MailboxSplit,
            PortConfig::PutPcMem32 => Type::MailboxShared,
            PortConfig::MailboxSplitOOB => Type::MailboxOobSplit,
            PortConfig::SlaveFlash => Type::BusMFlashS,
            PortConfig::MemSingle => Type::BusMMemS,
            PortConfig::MasterFlash => Type::BusMFlashS,
        }
    }
}

impl Default for PortConfig {
    fn default() -> Self {
        Self::Unconfigured
    }
}

/// eSPI capabilities.
#[derive(Clone, Copy)]
pub struct Capabilities {
    /// Mode of operation
    pub mode: Spicap,

    /// Max speed
    pub max_speed: Maxspd,

    /// Allow use of alert pin
    pub alert_as_a_pin: bool,

    /// Allow out-of-band
    pub allow_oob: bool,

    /// Allow 128b payload
    pub allow_128b_payload: bool,

    /// Flash payload size
    pub flash_payload_size: Flashmx,

    /// Slave-attached-flash erase size
    pub saf_erase_size: Option<Safera>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            mode: Spicap::Any,
            max_speed: Maxspd::SmallThan20m,
            alert_as_a_pin: false,
            allow_oob: false,
            allow_128b_payload: false,
            flash_payload_size: Flashmx::Byte64,
            saf_erase_size: None,
        }
    }
}

/// eSPI configuration.
#[derive(Clone, Copy)]
pub struct Config {
    /// Instance capabilities
    pub caps: Capabilities,

    /// Use 60MHz clock?
    pub use_60mhz: bool,

    /// RAM Base address
    pub ram_base: u32,

    /// Base 0 Address
    pub base0_addr: u32,

    /// Base 1 Address
    pub base1_addr: u32,

    /// Status Block address
    pub status_addr: Option<u16>,

    /// Status Block Base
    pub status_base: Base,

    /// Per-port configuration
    pub ports_config: [PortConfig; ESPI_PORTS],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            caps: Default::default(),
            use_60mhz: false,
            ram_base: 0,
            base0_addr: 0,
            base1_addr: 0,
            status_addr: None,
            status_base: Base::OffsetFrom0,
            ports_config: Default::default(),
        }
    }
}

/// Port event data
pub struct PortEvent {
    /// Offset accessed by Host
    pub offset: usize,

    /// Size of access
    pub length: usize,

    /// Direction of access
    pub direction: bool,
}

/// Wire Change Event
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WireChangeEvent {
    slp_s3n: bool,
    slp_s4n: bool,
    slp_s5n: bool,
    sus_stat: bool,
    pltrstn: bool,
    oob_rst_warn: bool,
    host_rst_warn: bool,
    sus_warnn: bool,
    sus_pwrdn_ackn: bool,
    slp_an: bool,
    slp_lann: bool,
    slp_wlann: bool,
    p2e: u8,
    host_c10n: bool,
}

impl WireChangeEvent {
    /// Set when power to non-critical systems should be shut off in
    /// S3 (Suspend to RAM).
    pub fn is_s3_sleep_control(&self) -> bool {
        self.slp_s3n
    }

    /// Set when power to non-critical systems should be shut off in
    /// S4 (Suspend to Disk).
    pub fn is_s4_sleep_control(&self) -> bool {
        self.slp_s4n
    }

    /// Set when power to non-critical systems should be shut off in
    /// S5 (Soft Off).
    pub fn is_s5_sleep_control(&self) -> bool {
        self.slp_s5n
    }

    /// Set when the system will be entering a low power state soon.
    pub fn is_suspend_status(&self) -> bool {
        self.sus_stat
    }

    /// Command to indicate Platform Reset assertion and de-assertion.
    pub fn is_platform_reset(&self) -> bool {
        self.pltrstn
    }

    /// Sent by controller just begore the OOB processor is about to
    /// enter reset.
    pub fn is_oob_reset_warn(&self) -> bool {
        self.oob_rst_warn
    }

    /// Sent by controller just before the Host is about to enter
    /// reset.
    pub fn is_host_reset_warn(&self) -> bool {
        self.host_rst_warn
    }

    /// Suspend about to happen.
    pub fn is_suspend_warn(&self) -> bool {
        self.sus_warnn
    }

    /// Host indicates that suspend power well can be shut down
    /// safely.
    pub fn is_suspend_power_down_ack(&self) -> bool {
        self.sus_pwrdn_ackn
    }

    /// Used when in Sx sleep but Management Engine is on.
    pub fn is_sleep_a(&self) -> bool {
        self.slp_an
    }

    /// Wired LAN can be powered down.
    pub fn is_sleep_lan(&self) -> bool {
        self.slp_lann
    }

    /// Wireless LAN can be powered down.
    pub fn is_sleep_wlan(&self) -> bool {
        self.slp_wlann
    }

    /// PCH to EC byte
    pub fn p2e(&self) -> u8 {
        self.p2e
    }

    /// Asserted when Host has entered deep power down state C10 or
    /// deeper.
    pub fn is_host_c10(&self) -> bool {
        self.host_c10n
    }
}

/// eSPI events.
pub enum Event {
    /// Port 0 has pending events
    Port0(PortEvent),

    /// Port 1 has pending events
    Port1(PortEvent),

    /// Port 2 has pending events
    Port2(PortEvent),

    /// Port 3 has pending events
    Port3(PortEvent),

    /// Port 4 has pending events
    Port4(PortEvent),

    /// Port 80 has pending events
    Port80,

    /// Change in virtual wires
    WireChange(WireChangeEvent),
}

/// eSPI Boot Status.
pub enum BootStatus {
    /// Success
    Success,

    /// Failure
    Failure,
}

impl From<BootStatus> for bool {
    fn from(status: BootStatus) -> bool {
        match status {
            BootStatus::Success => true,
            _ => false,
        }
    }
}

/// eSPI driver.
pub struct Espi<'d> {
    info: Info,
    _phantom: PhantomData<&'d ()>,
}

impl<'d> Espi<'d> {
    /// Instantiates new eSPI peripheral and initializes to default values.
    pub fn new<T: Instance>(
        _peripheral: impl Peripheral<P = T> + 'd,
        _clk: impl Peripheral<P = impl ClkPin<T>> + 'd,
        _cs: impl Peripheral<P = impl CsPin<T>> + 'd,
        _io0: impl Peripheral<P = impl Io0Pin<T>> + 'd,
        _io1: impl Peripheral<P = impl Io1Pin<T>> + 'd,
        _io2: impl Peripheral<P = impl Io2Pin<T>> + 'd,
        _io3: impl Peripheral<P = impl Io3Pin<T>> + 'd,
        _rst: impl Peripheral<P = impl RstPin<T>> + 'd,
        _alert: impl Peripheral<P = impl AlertPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Espi<'d> {
        into_ref!(_peripheral);
        into_ref!(_clk);
        into_ref!(_cs);
        into_ref!(_io0);
        into_ref!(_io1);
        into_ref!(_io2);
        into_ref!(_io3);
        into_ref!(_rst);
        into_ref!(_alert);

        _alert.as_alert();
        _rst.as_rst();
        _cs.as_cs();
        _io0.as_io0();
        _io1.as_io1();
        _clk.as_clk();
        _io2.as_io2();
        _io3.as_io3();

        // enable ESPI clock
        enable_and_reset::<T>();

        let mut instance = Espi::<'d> {
            info: T::info(),
            _phantom: PhantomData,
        };

        // Set ESPI mode
        instance.info.regs.mctrl().modify(|_, w| w.enable().espi());

        // Configure ports
        for port in 0..ESPI_PORTS {
            instance.configure(port, config.ports_config[port]);
        }

        // Set eSPI status block address
        if let Some(status_addr) = config.status_addr {
            // SAFETY: Unsafe only due to the use of `bits()`. All 16-bits are
            // valid, any 16-bit offset can be used.
            instance
                .info
                .regs
                .stataddr()
                .write(|w| unsafe { w.off().bits(status_addr) }.base().variant(config.status_base));

            instance.info.regs.mctrl().modify(|_, w| w.sblkena().set_bit());
        }

        // Set eSPI capabilities
        instance.info.regs.espicap().write(|w| {
            w.spicap()
                .variant(config.caps.mode)
                .maxspd()
                .variant(config.caps.max_speed)
                .alpin()
                .variant(config.caps.alert_as_a_pin)
                .oobok()
                .variant(config.caps.allow_oob)
                .memmx()
                .variant(config.caps.allow_128b_payload)
                .flashmx()
                .variant(config.caps.flash_payload_size)
                .saf()
                .variant(config.caps.saf_erase_size.is_some())
                .safera()
                .variant(config.caps.saf_erase_size.unwrap_or(Safera::Min2kb))
        });

        // Enable power save
        instance.info.regs.espimisc().write(|w| w.pwrsav().set_bit());

        // Clear Bus Reset status
        instance.info.regs.mstat().write(|w| w.bus_rst().clear_bit_by_one());

        // Set RAMBASE
        instance
            .info
            .regs
            .rambase()
            .write(|w| unsafe { w.bits(config.ram_base) });

        // Set MapBase addr
        instance.info.regs.mapbase().write(|w| unsafe {
            w.base1()
                .bits((config.base1_addr >> 16) as u16)
                .base0()
                .bits((config.base0_addr >> 16) as u16)
        });

        instance
            .info
            .regs
            .mctrl()
            .modify(|_, w| w.use60mhz().variant(config.use_60mhz));

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    /// Configure the port to a given mode
    pub fn configure(&mut self, port: usize, config: PortConfig) {
        match config {
            PortConfig::AcpiEndpoint { direction, addr } => {
                self.acpi_endpoint(port, direction, addr);
            }

            PortConfig::MailboxShared {
                direction,
                addr,
                offset,
                length,
            } => {
                self.mailbox(port, config.into(), direction, addr, offset, length);
            }

            PortConfig::MailboxSingle {
                direction,
                addr,
                offset,
                length,
            } => {
                self.mailbox(port, config.into(), direction, addr, offset, length);
            }

            PortConfig::MailboxSplit {
                direction,
                addr,
                offset,
                length,
            } => {
                self.mailbox(port, config.into(), direction, addr, offset, length);
            }

            _ => {
                self.info.regs.mctrl().modify(|_, w| w.pena(port as u8).disabled());
            }
        }
    }

    /// Complete port status
    pub async fn complete_port(&mut self, port: usize) {
        self.info.regs.port(port).stat().write(|w| {
            w.interr()
                .clear_bit_by_one()
                .intrd()
                .clear_bit_by_one()
                .intwr()
                .clear_bit_by_one()
                .intspc0()
                .clear_bit_by_one()
                .intspc1()
                .clear_bit_by_one()
                .intspc2()
                .clear_bit_by_one()
                .intspc3()
                .clear_bit_by_one()
        });

        // REVISIT: it's unclear if this is really needed, but it sure
        // helps getting things working.
        self.info.regs.port(port).irulestat().write(|w| w.srst().set_bit());
    }

    /// Wait for controller event
    pub async fn wait_for_event(&mut self) -> Result<Event> {
        self.wait_for(
            |me| {
                if me.info.regs.mstat().read().port_int0().bit_is_set() {
                    let datain = self.info.regs.port(0).datain().read();
                    let offset = datain.idx().bits() as usize;
                    let length = datain.data_len().bits() as usize + 1;
                    let direction = datain.dir().bit_is_set();

                    Poll::Ready(Ok(Event::Port0(PortEvent {
                        offset,
                        length,
                        direction,
                    })))
                } else if me.info.regs.mstat().read().port_int1().bit_is_set() {
                    let datain = self.info.regs.port(1).datain().read();
                    let offset = datain.idx().bits() as usize;
                    let length = datain.data_len().bits() as usize + 1;
                    let direction = datain.dir().bit_is_set();

                    Poll::Ready(Ok(Event::Port1(PortEvent {
                        offset,
                        length,
                        direction,
                    })))
                } else if me.info.regs.mstat().read().port_int2().bit_is_set() {
                    let datain = self.info.regs.port(2).datain().read();
                    let offset = datain.idx().bits() as usize;
                    let length = datain.data_len().bits() as usize + 1;
                    let direction = datain.dir().bit_is_set();

                    Poll::Ready(Ok(Event::Port2(PortEvent {
                        offset,
                        length,
                        direction,
                    })))
                } else if me.info.regs.mstat().read().port_int3().bit_is_set() {
                    let datain = self.info.regs.port(3).datain().read();
                    let offset = datain.idx().bits() as usize;
                    let length = datain.data_len().bits() as usize + 1;
                    let direction = datain.dir().bit_is_set();

                    Poll::Ready(Ok(Event::Port3(PortEvent {
                        offset,
                        length,
                        direction,
                    })))
                } else if me.info.regs.mstat().read().port_int4().bit_is_set() {
                    let datain = self.info.regs.port(4).datain().read();
                    let offset = datain.idx().bits() as usize;
                    let length = datain.data_len().bits() as usize + 1;
                    let direction = datain.dir().bit_is_set();

                    Poll::Ready(Ok(Event::Port4(PortEvent {
                        offset,
                        length,
                        direction,
                    })))
                } else if me.info.regs.mstat().read().p80int().bit_is_set() {
                    Poll::Ready(Ok(Event::Port80))
                } else if me.info.regs.mstat().read().wire_chg().bit_is_set() {
                    me.info.regs.mstat().write(|w| w.wire_chg().clear_bit_by_one());

                    let wirero = me.info.regs.wirero().read();

                    let event = WireChangeEvent {
                        slp_s3n: wirero.slp_s3n().bit_is_set(),
                        slp_s4n: wirero.slp_s4n().bit_is_set(),
                        slp_s5n: wirero.slp_s5n().bit_is_set(),
                        sus_stat: wirero.sus_stat().bit_is_set(),
                        pltrstn: wirero.pltrstn().bit_is_set(),
                        oob_rst_warn: wirero.oob_rst_warn().bit_is_set(),
                        host_rst_warn: wirero.host_rst_warn().bit_is_set(),
                        sus_warnn: wirero.sus_warnn().bit_is_set(),
                        sus_pwrdn_ackn: wirero.sus_pwrdn_ackn().bit_is_set(),
                        slp_an: wirero.slp_an().bit_is_set(),
                        slp_lann: wirero.slp_lann().bit_is_set(),
                        slp_wlann: wirero.slp_wlann().bit_is_set(),
                        p2e: wirero.p2e().bits(),
                        host_c10n: wirero.host_c10n().bit_is_set(),
                    };

                    Poll::Ready(Ok(Event::WireChange(event)))
                } else if me.info.regs.mstat().read().crcerr().bit_is_set() {
                    me.info.regs.mstat().write(|w| w.crcerr().clear_bit_by_one());
                    Poll::Ready(Err(Error::Crc))
                } else if me.info.regs.mstat().read().hstall().bit_is_set() {
                    me.info.regs.mstat().write(|w| w.hstall().clear_bit_by_one());
                    Poll::Ready(Err(Error::HStall))
                } else {
                    Poll::Pending
                }
            },
            |me| {
                me.info.regs.intenset().write(|w| {
                    w.port_int0()
                        .set_bit()
                        .port_int1()
                        .set_bit()
                        .port_int2()
                        .set_bit()
                        .port_int3()
                        .set_bit()
                        .port_int4()
                        .set_bit()
                        .p80int()
                        .set_bit()
                        .wire_chg()
                        .set_bit()
                        .hstall()
                        .set_bit()
                        .crcerr()
                        .set_bit()
                });
            },
        )
        .await
    }

    /// Wait for bus reset
    pub async fn wait_for_reset(&mut self) {
        self.wait_for(
            |me| {
                if me.info.regs.mstat().read().in_rst().bit_is_set() {
                    me.info.regs.mstat().write(|w| w.bus_rst().clear_bit_by_one());
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            },
            |me| {
                me.info.regs.intenset().write(|w| w.bus_rst().set_bit());
            },
        )
        .await
    }

    /// Push IRQ to Host
    pub async fn irq_push(&mut self, irq: u8) {
        self.info.regs.irqpush().write(|w| unsafe { w.irq().bits(irq) });

        self.wait_for(
            |me| {
                if me.info.regs.mstat().read().irq_upd().bit_is_set() {
                    me.info.regs.mstat().write(|w| w.irq_upd().clear_bit_by_one());
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            },
            |me| {
                me.info.regs.intenset().write(|w| w.irq_upd().set_bit());
            },
        )
        .await
    }

    /// Acknowledge OOB Reset.
    ///
    /// Active High.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn oob_reset_ack(&mut self) {
        self.info.regs.wirewo().write(|w| w.oob_rst_ack().set_bit());
        self.block_for_vwire_done();
    }

    /// Generate WAKE# event to wake Host up from Sx on any
    /// event. Also a general purpose event to wake on Lid switch or
    /// AC insertion.
    ///
    /// If the event occurs while Host is in S0, an SCI is generated
    /// instead.
    ///
    /// Active Low.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn wake(&mut self) {
        self.info.regs.wirewo().write(|w| w.waken_scin().clear_bit());
        self.block_for_vwire_done();
    }

    /// Generate PME# event to wake the Host from Sx through PCI PME#.
    ///
    /// Active Low.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn pme(&mut self) {
        self.info.regs.wirewo().write(|w| w.pmen().clear_bit());
        self.block_for_vwire_done();
    }

    /// Generate SCI event resulting in ACPI method being invoked by
    /// the OS.
    ///
    /// Active Low.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn sci(&mut self) {
        self.info.regs.wirewo().write(|w| w.scin().clear_bit());
        self.block_for_vwire_done();
    }

    /// Generate SMI event resulting in SMI code being invoked by the
    /// BIOS.
    ///
    /// Active Low.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn smi(&mut self) {
        self.info.regs.wirewo().write(|w| w.smin().clear_bit());
        self.block_for_vwire_done();
    }

    /// Generate RCIN event.
    ///
    /// Active Low.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn rcin(&mut self) {
        self.info.regs.wirewo().write(|w| w.rcinn().clear_bit());
        self.block_for_vwire_done();
    }

    /// Acknowledge Host Reset. Used in response to HOST_RST_WARN.
    ///
    /// Active High
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn host_reset_ack(&mut self) {
        self.info.regs.wirewo().write(|w| w.host_rst_ack().set_bit());
        self.block_for_vwire_done();
    }

    /// Acknowledge Suspend Warn.
    ///
    /// Active Low.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn suspend_ack(&mut self) {
        self.info.regs.wirewo().write(|w| w.susackn().clear_bit());
        self.block_for_vwire_done();
    }

    /// EC to PCH byte.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn e2p(&mut self, data: u8) {
        self.info.regs.wirewo().write(|w| unsafe { w.e2p().bits(data) });
        self.block_for_vwire_done();
    }

    /// Sent when EC or BMC has completed its boot process as an
    /// indication to eSPI controller to continue with G3 to S0 exit.
    ///
    /// Active High.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn boot_done(&mut self) {
        self.info.regs.wirewo().write(|w| w.boot_done().set_bit());
        self.block_for_vwire_done();
    }

    /// If boot ended in success, set to `true`.
    ///
    /// Active High.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn boot_status(&mut self, status: BootStatus) {
        self.info.regs.wirewo().write(|w| w.boot_errn().variant(status.into()));
        self.block_for_vwire_done();
    }

    /// To be called when Host goes into G3.
    ///
    /// Active High.
    ///
    /// Warning: Blocks until DONE bit clears
    pub fn dsw_pwrok_reset(&mut self) {
        self.info.regs.wirewo().write(|w| w.dsw_pwrok_rst().set_bit());
        self.block_for_vwire_done();
    }

    fn block_for_vwire_done(&self) {
        // No interrupt event available, must busy loop
        while self.info.regs.wirewo().read().done().bit_is_clear() {}
    }

    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once the waker is set (to eg enable the required interrupts).
    async fn wait_for<F, U, G>(&mut self, mut f: F, mut g: G) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        poll_fn(|cx| {
            let r = f(self);

            if r.is_pending() {
                ESPI_WAKER.register(cx.waker());
                g(self);
            }

            r
        })
        .await
    }
}

impl Espi<'_> {
    fn acpi_endpoint(&mut self, port: usize, direction: Direction, addr: u16) {
        self.info
            .regs
            .port(port)
            .cfg()
            .write(|w| w.type_().acpi_end().direction().variant(direction));

        // Set port interrupt rules
        self.info.regs.port(port).irulestat().write(|w| {
            unsafe { w.ustat().bits(0x1b) }
                .interr()
                .set_bit()
                .intrd()
                .set_bit()
                .intwr()
                .set_bit()
                .intspc0()
                .set_bit()
                .intspc1()
                .set_bit()
                .intspc2()
                .set_bit()
                .intspc3()
                .set_bit()
        });

        // Set port mapped address
        self.info
            .regs
            .port(port)
            .addr()
            .write(|w| unsafe { w.off().bits(addr) });

        // Enable the port
        self.info.regs.mctrl().modify(|_, w| w.pena(port as u8).enabled());

        // write 0x44 to data out
        self.info
            .regs
            .port(port)
            .dataout()
            .write(|w| unsafe { w.data().bits(0x44) });
    }

    fn mailbox(&mut self, port: usize, port_type: Type, direction: Direction, addr: u16, offset: u16, length: Len) {
        // Set port type
        self.info
            .regs
            .port(port)
            .cfg()
            .modify(|_, w| w.type_().variant(port_type));

        // Set port direction
        self.info
            .regs
            .port(port)
            .cfg()
            .modify(|_, w| w.direction().variant(direction));

        // Set port interrupt rules
        self.info.regs.port(port).irulestat().write(|w| {
            unsafe { w.ustat().bits(0) }
                .interr()
                .set_bit()
                .intrd()
                .set_bit()
                .intwr()
                .set_bit()
                .intspc0()
                .set_bit()
                .intspc1()
                .set_bit()
                .intspc2()
                .set_bit()
                .intspc3()
                .set_bit()
        });

        // Set port mapped address
        self.info
            .regs
            .port(port)
            .addr()
            .write(|w| unsafe { w.off().bits(addr) });

        // Set port RAM use
        self.info
            .regs
            .port(port)
            .ramuse()
            .write(|w| unsafe { w.off().bits(offset) }.len().variant(length));

        // Enable the port
        self.info.regs.mctrl().modify(|_, w| w.pena(port as u8).enabled());
    }
}

#[derive(Clone, Copy)]
struct Info {
    regs: &'static crate::pac::espi::RegisterBlock,
}

trait SealedInstance {
    fn info() -> Info;
}

/// eSPI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + SysconPeripheral + 'static + Send {
    /// Interrupt for this eSPI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl Instance for peripherals::ESPI {
    type Interrupt = crate::interrupt::typelevel::ESPI;
}

impl SealedInstance for peripherals::ESPI {
    fn info() -> Info {
        Info {
            // SAFETY: safe from single executor
            regs: unsafe { &*crate::pac::Espi::ptr() },
        }
    }
}

trait SealedPin: Pin {
    fn as_espi(&self, function: Function) {
        self.set_function(function)
            .set_pull(Pull::Up)
            .enable_input_buffer()
            .set_slew_rate(SlewRate::Standard)
            .set_drive_strength(DriveStrength::Normal)
            .disable_analog_multiplex()
            .set_drive_mode(DriveMode::PushPull)
            .set_input_inverter(Inverter::Disabled);
    }
}

macro_rules! pin_traits {
    ($mode:ident, $($pin:ident, $function:ident),*) => {
        paste! {
            /// Select pin mode of operation
            #[allow(private_bounds)]
            pub trait [<$mode:camel Pin>]<T: Instance>: SealedPin + crate::Peripheral {
                /// Set pin mode of operation
                fn [<as_ $mode>](&self);
            }
        }

	$(
	    paste!{
		impl SealedPin for crate::peripherals::$pin {}

		impl [<$mode:camel Pin>]<crate::peripherals::ESPI> for crate::peripherals::$pin {
		    fn [<as_ $mode>](&self) {
			self.as_espi(Function::$function);
		    }
		}
	    }
	)*
    };
}

pin_traits!(alert, PIO7_24, F6);
pin_traits!(rst, PIO7_25, F6);
pin_traits!(cs, PIO7_26, F6);
pin_traits!(io0, PIO7_27, F6);
pin_traits!(io1, PIO7_28, F6);
pin_traits!(clk, PIO7_29, F6);
pin_traits!(io2, PIO7_30, F6);
pin_traits!(io3, PIO7_31, F6);
