//! DMA

pub mod channel;
pub mod util;

use core::ptr;

use crate::dma::channel::{Channel, ChannelAndRequest, Request};
use crate::{interrupt, peripherals, Peripheral};
use embassy_hal_internal::{into_ref, PeripheralRef};

// TODO:
//  - add support for DMA1
//  - configure NVIC table

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
    }

    // TODO - return Result
    /// Reserve DMA channel
    pub fn reserve_channel(&mut self, channel: u8) -> ChannelAndRequest<'d, T> {
        let request: Request = 0; // TODO

        let channel = Channel {
            controller: unsafe { self.inner.clone_unchecked() }, // TODO - better design option?
            number: channel,
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
