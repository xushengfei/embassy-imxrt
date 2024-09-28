//! DMA

pub mod channel;
pub mod transfer;

use crate::dma::channel::{Channel, ChannelAndRequest, Request};
use crate::{interrupt, peripherals, Peripheral};
use core::ptr;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

// TODO:
//  - add support for DMA1
//  - configure NVIC table

/// DMA channel ID
pub enum ChannelId {
    /// Channel ID 0
    Channel0,
    /// Channel ID 1
    Channel1,
    /// Channel ID 2
    Channel2,
    /// Channel ID 3
    Channel3,
    /// Channel ID 4
    Channel4,
    /// Channel ID 5
    Channel5,
    /// Channel ID 6
    Channel6,
    /// Channel ID 7
    Channel7,
    /// Channel ID 8
    Channel8,
    /// Channel ID 9
    Channel9,
    /// Channel ID 10
    Channel10,
    /// Channel ID 11
    Channel11,
    /// Channel ID 12
    Channel12,
    /// Channel ID 13
    Channel13,
    /// Channel ID 14
    Channel14,
    /// Channel ID 15
    Channel15,
    /// Channel ID 16
    Channel16,
    /// Channel ID 17
    Channel17,
    /// Channel ID 18
    Channel18,
    /// Channel ID 19
    Channel19,
    /// Channel ID 20
    Channel20,
    /// Channel ID 21
    Channel21,
    /// Channel ID 22
    Channel22,
    /// Channel ID 23
    Channel23,
    /// Channel ID 24
    Channel24,
    /// Channel ID 25
    Channel25,
    /// Channel ID 26
    Channel26,
    /// Channel ID 27
    Channel27,
    /// Channel ID 28
    Channel28,
    /// Channel ID 29
    Channel29,
    /// Channel ID 30
    Channel30,
    /// Channel ID 31
    Channel31,
    /// Channel ID 32
    Channel32,
}

impl From<ChannelId> for usize {
    fn from(channel_id: ChannelId) -> Self {
        match channel_id {
            ChannelId::Channel0 => 0,
            ChannelId::Channel1 => 1,
            ChannelId::Channel2 => 2,
            ChannelId::Channel3 => 3,
            ChannelId::Channel4 => 4,
            ChannelId::Channel5 => 5,
            ChannelId::Channel6 => 6,
            ChannelId::Channel7 => 7,
            ChannelId::Channel8 => 8,
            ChannelId::Channel9 => 9,
            ChannelId::Channel10 => 10,
            ChannelId::Channel11 => 11,
            ChannelId::Channel12 => 12,
            ChannelId::Channel13 => 13,
            ChannelId::Channel14 => 14,
            ChannelId::Channel15 => 15,
            ChannelId::Channel16 => 16,
            ChannelId::Channel17 => 17,
            ChannelId::Channel18 => 18,
            ChannelId::Channel19 => 19,
            ChannelId::Channel20 => 20,
            ChannelId::Channel21 => 21,
            ChannelId::Channel22 => 22,
            ChannelId::Channel23 => 23,
            ChannelId::Channel24 => 24,
            ChannelId::Channel25 => 25,
            ChannelId::Channel26 => 26,
            ChannelId::Channel27 => 27,
            ChannelId::Channel28 => 28,
            ChannelId::Channel29 => 29,
            ChannelId::Channel30 => 30,
            ChannelId::Channel31 => 31,
            ChannelId::Channel32 => 32,
        }
    }
}

/// DMA channel descriptor
#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ChannelDescriptor {
    reserved: u32,
    src_data_end_addr: u32,
    dst_data_end_addr: u32,
    nxt_desc_link_addr: u32,
}

/// DMA channel descriptor memory block (1KB aligned)
#[repr(align(1024))]
#[derive(Copy, Clone, Debug)]
struct DescriptorBlock {
    list: [ChannelDescriptor; 33],
}

/// DMA channel descriptor list
static mut DESCRIPTORS: DescriptorBlock = DescriptorBlock {
    list: [ChannelDescriptor {
        reserved: 0,
        src_data_end_addr: 0,
        dst_data_end_addr: 0,
        nxt_desc_link_addr: 0,
    }; 33],
};

/// Error information type
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// configuration requested is not supported
    UnsupportedConfiguration,
}

#[allow(clippy::declare_interior_mutable_const)]
const DMA_WAKER: AtomicWaker = AtomicWaker::new();
const CHANNEL_COUNT: usize = 32;

// One waker per channel
static DMA_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [DMA_WAKER; CHANNEL_COUNT];

#[interrupt]
#[allow(non_snake_case)]
fn DMA0() {
    irq_handler(&DMA_WAKERS);
}

fn irq_handler<const N: usize>(wakers: &[AtomicWaker; N]) {
    let reg = unsafe { crate::pac::Dma0::steal() };

    info!("DMA0 Interrupt!");

    // Error interrupt pending?
    if reg.intstat().read().activeerrint().bit() {
        // TODO
    }

    // Interrupt pending?
    if reg.intstat().read().activeint().bit() {
        for (channel, waker) in wakers.iter().enumerate() {
            // Go through all the channels to check which ones contributed to the interrupt
            if reg.inta0().read().bits() & (1 << channel) != 0 {
                // Clear the interrupt for this channel
                reg.inta0().write(|w| unsafe { w.ia().bits(1 << channel) });
                // TODO - Disable the channel interrupt?
                waker.wake();
            }
        }
    }
}
/// DMA driver
pub struct Dma<'d, T: Instance> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Dma<'d, T> {
    /// Create a new DMA driver.
    pub fn new(inner: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(inner);
        let dma = Self { inner };

        // TODO: move - will have multiple consumers calling new
        Self::init();

        dma
    }

    /// Initialize the DMA controller
    pub fn init() {
        let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };
        let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

        // Enable the DMA controller clock
        clkctl1.pscctl1_set().write(|w| w.dmac0_clk_set().set_bit());

        // Clear DMA reset
        rstctl1.prstctl1_clr().write(|w| w.dmac0_rst_clr().set_bit());

        // Enable DMA controller
        T::regs().ctrl().modify(|_, w| w.enable().set_bit());

        // Set channel descriptor SRAM base address
        // SAFETY: unsafe due to .bits usage and use of a mutable static (DESCRIPTORS.list)
        unsafe {
            // Descriptor base must be 1K aligned
            let descriptor_base = ptr::addr_of!(DESCRIPTORS.list) as u32;
            T::regs().srambase().write(|w| w.bits(descriptor_base));
        }

        // Ensure AHB priority it highest (M4 == DMAC0)
        // SAFETY: unsafe only due to .bits usage
        sysctl0.ahbmatrixprior().modify(|_, w| unsafe { w.m4().bits(0) });

        // Enable DMA interrupts on DMA0
        interrupt::DMA0.unpend();
        unsafe {
            interrupt::DMA0.enable();
        }
    }

    // TODO - return Result
    /// Reserve DMA channel
    pub fn reserve_channel(&mut self, channel: ChannelId) -> ChannelAndRequest<'d, T> {
        let request: Request = 0; // TODO

        let channel = Channel {
            controller: unsafe { self.inner.clone_unchecked() }, // TODO - better design option?
            number: channel.into(),
        };

        ChannelAndRequest { channel, request }
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::Dma0;
}

/// DMA instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + 'static + Send {
    /// Interrupt for this DMA instance
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl Instance for peripherals::DMA0 {
    type Interrupt = crate::interrupt::typelevel::DMA0;
}

impl SealedInstance for peripherals::DMA0 {
    fn regs() -> crate::pac::Dma0 {
        // SAFETY: safe from single executor
        unsafe { crate::pac::Dma0::steal() }
    }
}
