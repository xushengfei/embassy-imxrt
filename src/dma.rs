//! DMA
use core::ptr;

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
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Dma<'d, T> {
    /// Create a new DMA driver.
    pub fn new(inner: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(inner);
        let dma = Self { _inner: inner };

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

    /// Ready the specified DMA channel for triggering
    pub fn configure_channel(&mut self, channel: usize, src: &[u8], dst: &mut [u8]) -> Result<(), Error> {
        // TODO

        let length = core::cmp::max(src.len(), dst.len());

        let srcbase = src.as_ptr() as u32;
        let dstbase = dst.as_mut_ptr() as u32;

        let xfercount = length - 1;
        let xferwidth = 1;

        // Configure descriptor
        unsafe {
            DESCRIPTORS.list[channel].reserved = 0;
            DESCRIPTORS.list[channel].src_data_end_addr = srcbase + (xfercount * xferwidth) as u32;
            DESCRIPTORS.list[channel].dst_data_end_addr = dstbase + (xfercount * xferwidth) as u32;
            DESCRIPTORS.list[channel].nxt_desc_link_addr = 0;
        }

        // Configure for memory-to-memory, no HW trigger, high priority
        T::regs().channel(channel).cfg().modify(|_, w| unsafe {
            w.periphreqen().clear_bit();
            w.hwtrigen().clear_bit();
            w.chpriority().bits(0)
        });

        // Mark configuration valid, clear trigger on complete, width is 1 byte, source & destination increments are width x 1 (1 byte), no reload
        T::regs().channel(channel).xfercfg().modify(|_, w| unsafe {
            w.cfgvalid().set_bit();
            w.clrtrig().set_bit();
            w.reload().clear_bit();
            w.width().bits(0);
            w.srcinc().bits(1);
            w.dstinc().bits(1);
            w.xfercount().bits(xfercount as u16)
        });
        Ok(())
    }

    /// Enable the specified DMA channel (must be configured)
    pub fn enable_channel(&mut self, channel: usize) -> Result<(), Error> {
        // TODO
        T::regs()
            .enableset0()
            .modify(|_, w| unsafe { w.ena().bits(1 << channel) });
        Ok(())
    }
    /// Trigger the specified DMA channel
    pub fn trigger_channel(&mut self, channel: usize) -> Result<(), Error> {
        // TODO
        T::regs().channel(channel).xfercfg().modify(|_, w| w.swtrig().set_bit());
        Ok(())
    }

    /// Is the specified DMA channel active?
    pub fn is_channel_active(&mut self, channel: usize) -> Result<bool, Error> {
        // TODO
        Ok(T::regs().active0().read().act().bits() & (1 << channel) != 0)
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
