//! DMA utilities

use super::Instance;

use crate::dma::channel::{Channel, Request};

/// DMA transfer options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Request transfer data width
    pub width: Width,

    /// Request priority level
    pub priority: Priority,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            width: Width::Bit8,
            priority: Priority::Priority0,
        }
    }
}

/// DMA request priority.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Priority {
    /// Priority 7 (lowest)
    Priority7,
    /// Priority 6
    Priority6,
    /// Priority 5
    Priority5,
    /// Priority 4
    Priority4,
    /// Priority 3
    Priority3,
    /// Priority 2
    Priority2,
    /// Priority 1
    Priority1,
    /// Priority 0 (highest)
    Priority0,
}

/// DMA transfer width
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Width {
    /// 8-bit width
    Bit8,
    /// 16-bit width
    Bit16,
    /// 32-bit width
    Bit32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Dir {
    MemoryToMemory,
    MemoryToPeripheral,
    PeripheralToMemory,
}

/// DMA transfer.
//#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'d, T: Instance> {
    /// DMA channel
    pub _channel: &'d Channel<'d, T>, // TODO
}

// TODO - handle different word sizes

impl<'d, T: Instance> Transfer<'d, T> {
    /// Create a new read DMA transfer (peripheral to memory).
    pub fn new_read(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        peri_addr: *const u8, // TODO
        buf: &'d mut [u8],    // TODO
        options: TransferOptions,
    ) -> Self {
        Self::new_read_raw(channel, request, peri_addr, buf, options)
    }

    /// Create a new read DMA transfer (peripheral to memory), using raw pointers.
    pub fn new_read_raw(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        peri_addr: *const u8,
        buf: *mut [u8],
        options: TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel,
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut u8 as *mut u32, // TODO
            buf.len(),
            options,
        )
    }

    /// Create a new write DMA transfer (memory to peripheral).
    pub fn new_write(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        buf: &'d [u8],
        peri_addr: *mut u8,
        options: TransferOptions,
    ) -> Self {
        Self::new_write_raw(channel, request, buf, peri_addr, options)
    }

    /// Create a new write DMA transfer (memory to peripheral), using raw pointers.
    pub fn new_write_raw(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        buf: *const [u8],   // TODO
        peri_addr: *mut u8, // TODO
        options: TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel,
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *mut u32,
            buf as *const u8 as *mut u32, // TODO
            buf.len(),
            options,
        )
    }

    /// Create a new write DMA transfer (memory to memory).
    pub fn new_write_mem(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        src_buf: &'d [u8],
        dst_buf: &'d mut [u8],
        options: TransferOptions,
    ) -> Self {
        Self::new_write_mem_raw(channel, request, src_buf, dst_buf, options)
    }

    /// Create a new write DMA transfer (memory to memory), using raw pointers.
    pub fn new_write_mem_raw(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        src_buf: *const [u8], // TODO
        dst_buf: *mut [u8],   // TODO
        options: TransferOptions,
    ) -> Self {
        Self::new_inner_mem(
            channel,
            request,
            Dir::MemoryToMemory,
            src_buf as *const u32, // TODO
            dst_buf as *mut u32,
            src_buf.len(),
            options,
        )
    }

    fn new_inner(
        channel: &'d mut Channel<'d, T>,
        _request: Request,
        _dir: Dir,
        _peri_addr: *const u32,
        _buf: *mut u32,
        _mem_len: usize,
        _options: TransferOptions,
    ) -> Self {
        // 1. configure_channel
        // 2. enable_channel
        // 3. trigger_channel
        let a = channel.is_channel_active(channel.number as usize);
        info!("DMA channel active: {}", a);

        Self { _channel: channel }
    }

    fn new_inner_mem(
        channel: &'d mut Channel<'d, T>,
        _request: Request,
        _dir: Dir,
        src_buf: *const u32,
        dst_buf: *mut u32,
        mem_len: usize,
        _options: TransferOptions,
    ) -> Self {
        // 1. configure_channel
        match channel.configure_channel(channel.number as usize, src_buf, dst_buf, mem_len) {
            Ok(v) => v,
            Err(_e) => info!("failed to configure DMA channel"),
        };
        // 2. enable_channel
        match channel.enable_channel(channel.number as usize) {
            Ok(v) => v,
            Err(_e) => info!("failed to enable DMA channel"),
        };

        // 3. trigger_channel
        match channel.trigger_channel(channel.number as usize) {
            Ok(v) => v,
            Err(_e) => info!("failed to trigger DMA channel"),
        };

        Self { _channel: channel }
    }
}
