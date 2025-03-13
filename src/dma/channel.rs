//! DMA channel & request

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;

use super::{DESCRIPTORS, DMA_WAKERS};
use crate::dma::transfer::{Direction, Transfer, TransferOptions};
use crate::dma::DmaInfo;

/// DMA channel
pub struct Channel<'d> {
    /// DMA channel peripheral reference
    pub(super) info: DmaInfo,
    /// Keep track of lifetime for Channel within the DMA module
    pub(super) _lifetime: PhantomData<&'d ()>,
}

impl<'d> Channel<'d> {
    /// Reads from a peripheral into a memory buffer
    pub fn read_from_peripheral(
        &'d self,
        peri_addr: *const u8,
        buf: &'d mut [u8],
        options: TransferOptions,
    ) -> Transfer<'d> {
        Transfer::new_read(self, peri_addr, buf, options)
    }

    /// Writes from a memory buffer to a peripheral
    pub fn write_to_peripheral(&'d self, buf: &'d [u8], peri_addr: *mut u8, options: TransferOptions) -> Transfer<'d> {
        Transfer::new_write(self, buf, peri_addr, options)
    }

    /// Writes from a memory buffer to another memory buffer
    pub async fn write_to_memory(
        &'d self,
        src_buf: &'d [u8],
        dst_buf: &'d mut [u8],
        options: TransferOptions,
    ) -> Transfer<'d> {
        let transfer = Transfer::new_write_mem(self, src_buf, dst_buf, options);
        self.poll_transfer_complete().await;
        transfer
    }

    /// Return a reference to the channel's waker
    pub fn get_waker(&self) -> &'d AtomicWaker {
        &DMA_WAKERS[self.info.ch_num]
    }

    /// Check whether DMA is active
    pub fn is_active(&self) -> bool {
        let channel = self.info.ch_num;
        self.info.regs.active0().read().act().bits() & (1 << channel) != 0
    }

    /// Check whether DMA is busy
    pub fn is_busy(&self) -> bool {
        let channel = self.info.ch_num;
        self.info.regs.busy0().read().bsy().bits() & (1 << channel) != 0
    }

    /// Return DMA remaining transfer count
    /// To get number of bytes, do `(XFERCOUNT + 1) x data width`
    pub fn get_xfer_count(&self) -> u16 {
        let channel = self.info.ch_num;
        self.info.regs.channel(channel).xfercfg().read().xfercount().bits()
    }

    /// Abort DMA operation
    pub fn abort(&self) {
        let channel = self.info.ch_num;
        self.disable_channel();
        while self.is_busy() {}
        self.info.regs.abort0().write(|w|
            // SAFETY: unsafe due to .bits usage
            unsafe { w.bits(1 << channel) });
    }

    async fn poll_transfer_complete(&'d self) {
        poll_fn(|cx| {
            // TODO - handle transfer failure

            let channel = self.info.ch_num;

            // Has the transfer already completed?
            if self.info.regs.active0().read().act().bits() & (1 << channel) == 0 {
                return Poll::Ready(());
            }

            DMA_WAKERS[channel].register(cx.waker());

            // Has the transfer completed now?
            if self.info.regs.active0().read().act().bits() & (1 << channel) == 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Prepare the DMA channel for the transfer
    pub fn configure_channel(
        &self,
        dir: Direction,
        srcbase: *const u32,
        dstbase: *mut u32,
        mem_len: usize,
        options: TransferOptions,
    ) {
        if mem_len % options.width.byte_width() != 0 {
            panic!(
                "Memory length({}) must be a multiple of the transfer width({})",
                mem_len,
                options.width.byte_width()
            );
        }

        let xferwidth: usize = options.width.byte_width();
        let xfercount = (mem_len / xferwidth) - 1;
        let channel = self.info.ch_num;

        // Configure for transfer type, no hardware triggering (we'll trigger via software), high priority
        // SAFETY: unsafe due to .bits usage
        self.info.regs.channel(channel).cfg().write(|w| unsafe {
            if dir == Direction::MemoryToMemory {
                w.periphreqen().clear_bit();
            } else {
                w.periphreqen().set_bit();
            }
            w.hwtrigen().clear_bit();
            w.chpriority().bits(0)
        });

        // Enable the interrupt on this channel
        self.info
            .regs
            .intenset0()
            .write(|w| unsafe { w.inten().bits(1 << channel) });

        // Mark configuration valid, clear trigger on complete, width is 1 byte, source & destination increments are width x 1 (1 byte), no reload
        // SAFETY: unsafe due to .bits usage
        self.info.regs.channel(channel).xfercfg().write(|w| unsafe {
            w.cfgvalid().set_bit();

            if options.is_continuous {
                w.reload().enabled();
                w.clrtrig().clear_bit();
            } else {
                w.reload().disabled();
                w.clrtrig().set_bit();
            }
            if options.is_sw_trig {
                w.swtrig().set_bit();
            } else {
                w.swtrig().clear_bit();
            }
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

        // Configure the channel descriptor
        // NOTE: the DMA controller expects the memory buffer end address but peripheral address is actual
        // SAFETY: unsafe due to use of a mutable static (DESCRIPTORS.list)
        unsafe {
            if options.is_continuous {
                let xfer_cfg = self.info.regs.channel(channel).xfercfg().read();
                DESCRIPTORS.list[channel].reserved_reloadcfg = xfer_cfg.bits();
            } else {
                DESCRIPTORS.list[channel].reserved_reloadcfg = 0;
            }

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
            if options.is_continuous {
                DESCRIPTORS.list[channel].nxt_desc_link_addr = &DESCRIPTORS.list[channel] as *const _ as u32;
            } else {
                DESCRIPTORS.list[channel].nxt_desc_link_addr = 0;
            }
        }
    }

    /// Enable the DMA channel (only after configuring)
    // SAFETY: unsafe due to .bits usage
    pub fn enable_channel(&self) {
        let channel = self.info.ch_num;
        self.info
            .regs
            .enableset0()
            .modify(|_, w| unsafe { w.ena().bits(1 << channel) });
    }

    #[cfg(feature = "defmt")]
    /// Log the DMA channel transfer configuration
    pub fn log_channel(&self) {
        // Log DMA Channel Config Register
        let channel = self.info.ch_num;
        let dma_channel_cfg = self.info.regs.channel(channel).cfg().read();
        defmt::info!(
            "DMA Channel {} Config Register: 0x{=u32:x}",
            channel,
            dma_channel_cfg.bits()
        );
        // Log Interrupt Enable Set Register
        let intenset0 = self.info.regs.intenset0().read();
        defmt::info!("Interrupt Enable Set Register: 0x{=u32:x}", intenset0.bits());

        // Log DMA Xfer Config Register
        let xfer_cfg = self.info.regs.channel(channel).xfercfg().read();
        defmt::info!(
            "DMA Channel {} Xfer Config Register: 0x{=u32:x}",
            channel,
            xfer_cfg.bits()
        );
        // Log DMA Enable Set Register after enabling
        let enableset0 = self.info.regs.enableset0().read();
        defmt::info!("DMA Enable Set Register:0x{=u32:x}", enableset0.bits());
        // Log DMA Xfer Config Register after triggering
        let xfer_cfg_after = self.info.regs.channel(channel).xfercfg().read();
        defmt::info!(
            "DMA Channel {} Xfer Config Register (After Trigger): 0x{=u32:x}",
            channel,
            xfer_cfg_after.bits()
        );
    }

    /// Disable the DMA channel
    pub fn disable_channel(&self) {
        let channel = self.info.ch_num;
        self.info.regs.enableclr0().write(|w|
            // SAFETY: unsafe due to .bits usage
            unsafe { w.clr().bits(1 << channel) });
    }

    /// Trigger the DMA channel
    pub fn trigger_channel(&self) {
        let channel = self.info.ch_num;
        self.info
            .regs
            .channel(channel)
            .xfercfg()
            .modify(|_, w| w.swtrig().set_bit());
    }
}
