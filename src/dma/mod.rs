//! DMA

pub mod channel;
pub mod transfer;

use crate::dma::channel::{Channel, ChannelAndRequest, Request};
use crate::{interrupt, peripherals, Peripheral};
use core::ptr;
use embassy_hal_internal::{interrupt::InterruptExt, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

// TODO:
//  - add support for DMA1

const DMA_CHANNEL_COUNT: usize = 33;

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
    list: [ChannelDescriptor; DMA_CHANNEL_COUNT],
}

/// DMA channel descriptor list
static mut DESCRIPTORS: DescriptorBlock = DescriptorBlock {
    list: [ChannelDescriptor {
        reserved: 0,
        src_data_end_addr: 0,
        dst_data_end_addr: 0,
        nxt_desc_link_addr: 0,
    }; DMA_CHANNEL_COUNT],
};

/// DMA errors
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Configuration requested is not supported
    UnsupportedConfiguration,
}

#[allow(clippy::declare_interior_mutable_const)]
const DMA_WAKER: AtomicWaker = AtomicWaker::new();

// One waker per channel
static DMA_WAKERS: [AtomicWaker; DMA_CHANNEL_COUNT] = [DMA_WAKER; DMA_CHANNEL_COUNT];

#[interrupt]
#[allow(non_snake_case)]
fn DMA0() {
    irq_handler(&DMA_WAKERS);
}

fn irq_handler<const N: usize>(wakers: &[AtomicWaker; N]) {
    let reg = unsafe { crate::pac::Dma0::steal() };

    // Error interrupt pending?
    if reg.intstat().read().activeerrint().bit() {
        info!("DMA error interrupt!");
        // TODO
    }

    // DMA transfer completion interrupt pending?
    if reg.intstat().read().activeint().bit() {
        let ia = reg.inta0().read().bits();
        // Loop through interrupt bitfield, excluding trailing and leading zeros looking for interrupt source(s)
        for channel in ia.trailing_zeros()..(32 - ia.leading_zeros()) {
            if ia & (1 << channel) != 0 {
                info!("DMA interrupt on channel {}!", channel);
                // Clear the interrupt for this channel
                reg.inta0().write(|w| unsafe { w.ia().bits(1 << channel) });
                wakers[channel as usize].wake();
            }
        }
    }
}

/// Initialize the DMA controller
pub fn init() {
    let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };
    let rstctl1 = unsafe { crate::pac::Rstctl1::steal() };
    let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };
    let dmactl0 = unsafe { crate::pac::Dma0::steal() };

    // Enable the DMA controller clock
    clkctl1.pscctl1_set().write(|w| w.dmac0_clk_set().set_bit());

    // Clear DMA reset
    rstctl1.prstctl1_clr().write(|w| w.dmac0_rst_clr().set_bit());

    // Enable DMA controller
    dmactl0.ctrl().modify(|_, w| w.enable().set_bit());

    // Set channel descriptor SRAM base address
    // SAFETY: unsafe due to .bits usage and use of a mutable static (DESCRIPTORS.list)
    unsafe {
        // Descriptor base must be 1K aligned
        let descriptor_base = ptr::addr_of!(DESCRIPTORS.list) as u32;
        dmactl0.srambase().write(|w| w.bits(descriptor_base));
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

/// DMA device
pub struct Dma<'d, T: Instance> {
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Dma<'d, T> {
    /// Reserve DMA channel
    pub fn reserve_channel(channel: impl Peripheral<P = T> + 'd) -> ChannelAndRequest<'d, T> {
        let request: Request = 0; // TODO
        let channel = Channel {
            inner: channel.into_ref(),
        };

        ChannelAndRequest { channel, request }
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::Dma0;
    fn get_channel_number() -> usize;
}

/// DMA instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + 'static + Send {
    /// Interrupt for this DMA instance
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! dma_channel_instance {
    ($instance: ident, $controller: expr, $number: expr) => {
        impl Instance for peripherals::$instance {
            type Interrupt = crate::interrupt::typelevel::DMA0;
        }

        impl SealedInstance for peripherals::$instance {
            fn regs() -> crate::pac::Dma0 {
                // SAFETY: safe from single executor
                unsafe { crate::pac::Dma0::steal() }
            }
            fn get_channel_number() -> usize {
                $number
            }
        }
    };
}

dma_channel_instance!(DMA0_CH0, crate::pac::Dma0, 0);
dma_channel_instance!(DMA0_CH1, crate::pac::Dma0, 1);
dma_channel_instance!(DMA0_CH2, crate::pac::Dma0, 2);
dma_channel_instance!(DMA0_CH3, crate::pac::Dma0, 3);
dma_channel_instance!(DMA0_CH4, crate::pac::Dma0, 4);
dma_channel_instance!(DMA0_CH5, crate::pac::Dma0, 5);
dma_channel_instance!(DMA0_CH6, crate::pac::Dma0, 6);
dma_channel_instance!(DMA0_CH7, crate::pac::Dma0, 7);
dma_channel_instance!(DMA0_CH8, crate::pac::Dma0, 8);
dma_channel_instance!(DMA0_CH9, crate::pac::Dma0, 9);
dma_channel_instance!(DMA0_CH10, crate::pac::Dma0, 10);
dma_channel_instance!(DMA0_CH11, crate::pac::Dma0, 11);
dma_channel_instance!(DMA0_CH12, crate::pac::Dma0, 12);
dma_channel_instance!(DMA0_CH13, crate::pac::Dma0, 13);
dma_channel_instance!(DMA0_CH14, crate::pac::Dma0, 14);
dma_channel_instance!(DMA0_CH15, crate::pac::Dma0, 15);
dma_channel_instance!(DMA0_CH16, crate::pac::Dma0, 16);
dma_channel_instance!(DMA0_CH17, crate::pac::Dma0, 17);
dma_channel_instance!(DMA0_CH18, crate::pac::Dma0, 18);
dma_channel_instance!(DMA0_CH19, crate::pac::Dma0, 19);
dma_channel_instance!(DMA0_CH20, crate::pac::Dma0, 20);
dma_channel_instance!(DMA0_CH21, crate::pac::Dma0, 21);
dma_channel_instance!(DMA0_CH22, crate::pac::Dma0, 22);
dma_channel_instance!(DMA0_CH23, crate::pac::Dma0, 23);
dma_channel_instance!(DMA0_CH24, crate::pac::Dma0, 24);
dma_channel_instance!(DMA0_CH25, crate::pac::Dma0, 25);
dma_channel_instance!(DMA0_CH26, crate::pac::Dma0, 26);
dma_channel_instance!(DMA0_CH27, crate::pac::Dma0, 27);
dma_channel_instance!(DMA0_CH28, crate::pac::Dma0, 28);
dma_channel_instance!(DMA0_CH29, crate::pac::Dma0, 29);
dma_channel_instance!(DMA0_CH30, crate::pac::Dma0, 30);
dma_channel_instance!(DMA0_CH31, crate::pac::Dma0, 31);
dma_channel_instance!(DMA0_CH32, crate::pac::Dma0, 32);
