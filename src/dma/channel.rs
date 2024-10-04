//! DMA channel & request

use super::Error;
use super::Instance;
use super::DESCRIPTORS;
use super::DMA_WAKERS;

use core::future::poll_fn;
use core::task::Poll;

use crate::dma::transfer::{Direction, Transfer, TransferOptions};
use embassy_hal_internal::PeripheralRef;

use embassy_sync::waitqueue::AtomicWaker;

/// DMA request identifier
pub type Request = u8;

/// Convenience wrapper, contains a DMA channel and a request
pub struct ChannelAndRequest<'d, T: Instance> {
    /// DMA channel
    pub channel: Channel<'d, T>,
    /// DMA request
    pub request: Request,
}

impl<'d, T: Instance> ChannelAndRequest<'d, T> {
    /// Reads from a peripheral into a memory buffer
    pub async fn read_from_peripheral(
        &'d self,
        peri_addr: *const u8,
        buf: &'d mut [u8],
        options: TransferOptions,
    ) -> Transfer<'d, T> {
        let transfer = Transfer::new_read(&self.channel, self.request, peri_addr, buf, options);
        self.poll_transfer_complete().await;
        transfer
    }

    /// Writes from a memory buffer to a peripheral
    pub async fn write_to_peripheral(
        &'d self,
        buf: &'d [u8],
        peri_addr: *mut u8,
        options: TransferOptions,
    ) -> Transfer<'d, T> {
        let transfer = Transfer::new_write(&self.channel, self.request, buf, peri_addr, options);
        self.poll_transfer_complete().await;
        transfer
    }

    /// Writes from a memory buffer to another memory buffer
    pub async fn write_to_memory(
        &'d self,
        src_buf: &'d [u8],
        dst_buf: &'d mut [u8],
        options: TransferOptions,
    ) -> Transfer<'d, T> {
        let transfer = Transfer::new_write_mem(&self.channel, self.request, src_buf, dst_buf, options);
        self.poll_transfer_complete().await;
        transfer
    }

    /// Return a reference to the channel's waker
    pub fn get_waker() -> &'d AtomicWaker {
        &DMA_WAKERS[T::get_channel_number()]
    }

    async fn poll_transfer_complete(&'d self) {
        poll_fn(|cx| {
            // TODO - handle transfer failure

            let channel = T::get_channel_number();

            // Has the transfer already completed?  If so the channel interrupt will be disabled (masked)
            if T::regs().intenset0().read().inten().bits() & (1 << channel) == 0 {
                // Enable the interrupt on this channel
                // SAFETY: unsafe due to .bits usage
                T::regs().intenset0().write(|w| unsafe { w.inten().bits(1 << channel) });
                return Poll::Ready(());
            }

            DMA_WAKERS[channel].register(cx.waker());

            // Has the transfer completed now?  If so the channel interrupt will be disabled (masked)
            if T::regs().intenset0().read().inten().bits() & (1 << channel) == 0 {
                // Enable the interrupt on this channel
                // SAFETY: unsafe due to .bits usage
                T::regs().intenset0().write(|w| unsafe { w.inten().bits(1 << channel) });
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

/// DMA channel
pub struct Channel<'d, T: Instance> {
    /// DMA channel peripheral reference
    pub inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Channel<'d, T> {
    /// Prepare the DMA channel for the transfer
    pub fn configure_channel(
        &self,
        dir: Direction,
        srcbase: *const u32,
        dstbase: *mut u32,
        mem_len: usize,
        options: TransferOptions,
    ) -> Result<(), Error> {
        let xfercount = mem_len - 1;
        let xferwidth = 1;
        let channel = T::get_channel_number();

        // Configure the channel descriptor
        // NOTE: the DMA controller expects the memory buffer end address but peripheral address is actual
        unsafe {
            DESCRIPTORS.list[channel].reserved = 0;
            if dir == Direction::MemoryToPeripheral {
                DESCRIPTORS.list[channel].dst_data_end_addr = dstbase as u32;
            } else {
                DESCRIPTORS.list[channel].dst_data_end_addr = dstbase as u32 + (xfercount * xferwidth) as u32;
            }
            if dir == Direction::PeripheralToMemory {
                DESCRIPTORS.list[channel].src_data_end_addr = srcbase as u32;
            } else {
                DESCRIPTORS.list[channel].src_data_end_addr = srcbase as u32 + (xfercount * xferwidth) as u32;
            }
            DESCRIPTORS.list[channel].nxt_desc_link_addr = 0;
        }

        // Configure for transfer type, no hardware triggering (we'll trigger via software), high priority
        T::regs().channel(channel).cfg().modify(|_, w| unsafe {
            if dir == Direction::MemoryToMemory {
                w.periphreqen().clear_bit();
            } else {
                w.periphreqen().set_bit();
            }
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

    /// Enable the DMA channel (only after configuring)
    pub fn enable_channel(&self) -> Result<(), Error> {
        let channel = T::get_channel_number();
        T::regs()
            .enableset0()
            .modify(|_, w| unsafe { w.ena().bits(1 << channel) });
        Ok(())
    }
    /// Trigger the DMA channel
    pub fn trigger_channel(&self) -> Result<(), Error> {
        let channel = T::get_channel_number();
        T::regs().channel(channel).xfercfg().modify(|_, w| w.swtrig().set_bit());
        Ok(())
    }
}
