//! DMA
use core::ptr;

use crate::{interrupt, peripherals, Peripheral};
use embassy_hal_internal::{into_ref, PeripheralRef};

// TODO:
//  - add support for DMA1
//  - configure NVIC table

/// DMA channel descriptor
#[derive(Copy, Clone, Debug)]
struct ChannelDescriptor {
    _reserved: u32,
    _src_data_end_addr: u32,
    _dst_data_end_addr: u32,
    _nxt_desc_link_addr: u32,
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
        _reserved: 0,
        _src_data_end_addr: 0,
        _dst_data_end_addr: 0,
        _nxt_desc_link_addr: 0,
    }; 33],
};

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
        info!("DMA init");

        let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };
        let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

        // Enable the DMA controller clock
        clkctl1.pscctl1().modify(|_, w| w.dmac0_clk().set_bit());

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
        unsafe {
            sysctl0.ahbmatrixprior().modify(|_, w| w.m4().bits(0));
        }
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
