//! DMA channel
use core::borrow::BorrowMut;

use super::Error;
use super::Instance;
use super::DESCRIPTORS;

use crate::dma::transfer::{Direction, Transfer, TransferOptions};
use embassy_hal_internal::PeripheralRef;

/// DMA request identifier
pub type Request = u8;

/// Convenience wrapper, contains a DMA controller, channel number and a request number.
///
/// Commonly used in peripheral drivers that own DMA channels.
///
pub struct ChannelAndRequest<'d, T: Instance> {
    /// DMA channel
    pub channel: Channel<'d, T>,
    /// DMA request
    pub request: Request,
}

impl<'d, T: Instance> ChannelAndRequest<'d, T> {
    /// Issues channel read request
    pub fn read(
        &'d mut self,
        peri_addr: *mut u8, // TODO
        buf: &'d mut [u8],  // TODO
        options: TransferOptions,
    ) -> Transfer<'d, T> {
        Transfer::new_read(self.channel.borrow_mut(), self.request, peri_addr, buf, options)
        // TODO
    }

    /// Issues channel write request
    pub fn write(
        &'d mut self,
        buf: &'d [u8], // TODO
        peri_addr: *mut u8,
        options: TransferOptions,
    ) -> Transfer<'d, T> {
        Transfer::new_write(self.channel.borrow_mut(), self.request, buf, peri_addr, options)
        // TODO
    }

    /// Issues channel write to memory (memory-to-memory) request
    pub fn write_mem(
        &'d mut self,
        src_buf: &'d [u8],     // TODO
        dst_buf: &'d mut [u8], // TODO
        options: TransferOptions,
    ) -> Transfer<'d, T> {
        Transfer::new_write_mem(self.channel.borrow_mut(), self.request, src_buf, dst_buf, options)
        // TODO
    }
}

/// DMA channel
pub struct Channel<'d, T: Instance> {
    /// DMA controller
    pub controller: PeripheralRef<'d, T>,
    /// DMA channel number
    pub number: usize,
}

impl<'d, T: Instance> Channel<'d, T> {
    /// Ready the specified DMA channel for triggering
    pub fn configure_channel(
        &mut self,
        channel: usize,
        dir: Direction,
        srcbase: *const u32,
        dstbase: *mut u32,
        mem_len: usize,
        options: TransferOptions,
    ) -> Result<(), Error> {
        let xfercount = mem_len - 1;
        let xferwidth = 1;

        // Configure descriptor
        unsafe {
            DESCRIPTORS.list[channel].reserved = 0;
            DESCRIPTORS.list[channel].src_data_end_addr = srcbase as u32 + (xfercount * xferwidth) as u32;
            DESCRIPTORS.list[channel].dst_data_end_addr = dstbase as u32 + (xfercount * xferwidth) as u32;
            DESCRIPTORS.list[channel].nxt_desc_link_addr = 0;
        }

        // Configure for memory-to-memory, no HW trigger, high priority
        T::regs().channel(channel).cfg().modify(|_, w| unsafe {
            w.periphreqen().clear_bit();
            w.hwtrigen().clear_bit();
            w.chpriority().bits(0)
        });

        // Enable the interrupt on this channel
        T::regs().intenset0().write(|w| unsafe { w.inten().bits(1 << channel) });

        // Mark configuration valid, clear trigger on complete, width is 1 byte, source & destination increments are width x 1 (1 byte), no reload
        T::regs().channel(channel).xfercfg().modify(|_, w| unsafe {
            w.cfgvalid().set_bit();
            w.clrtrig().set_bit();
            w.reload().clear_bit();
            w.setinta().set_bit();
            w.width().bits(options.width.into());
            if dir == Direction::PeripheralToMemory {
                w.srcinc().bits(0);
            } else {
                w.srcinc().bits(1);
            }
            if dir == Direction::MemoryToPeripheral {
                w.dstinc().bits(0);
            } else {
                w.dstinc().bits(1);
            }
            w.xfercount().bits(xfercount as u16)
        });
        Ok(())
    }

    /// Enable the specified DMA channel (must be configured)
    pub fn enable_channel(&mut self, channel: usize) -> Result<(), Error> {
        T::regs()
            .enableset0()
            .modify(|_, w| unsafe { w.ena().bits(1 << channel) });
        Ok(())
    }
    /// Trigger the specified DMA channel
    pub fn trigger_channel(&mut self, channel: usize) -> Result<(), Error> {
        T::regs().channel(channel).xfercfg().modify(|_, w| w.swtrig().set_bit());
        Ok(())
    }

    /// Is the specified DMA channel active?
    pub fn is_channel_active(&mut self, channel: u8) -> Result<bool, Error> {
        Ok(T::regs().active0().read().act().bits() & (1 << channel) != 0)
    }
}
