//! DMA transfer management

use super::Instance;

use crate::dma::channel::{Channel, Request};

/// DMA transfer options
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Transfer data width
    pub width: Width,

    /// Transfer priority level
    pub priority: Priority,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            width: Width::Bit32,
            priority: Priority::Priority0,
        }
    }
}

/// DMA transfer priority
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Priority {
    /// Priority 7 (lowest priority)
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
    /// Priority 0 (highest priority)
    Priority0,
}

/// DMA transfer width
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Width {
    /// 8 bits
    Bit8,
    /// 16 bits
    Bit16,
    /// 32 bits
    Bit32,
}

impl From<Width> for u8 {
    fn from(w: Width) -> Self {
        match w {
            Width::Bit8 => 0,
            Width::Bit16 => 1,
            Width::Bit32 => 2,
        }
    }
}

/// DMA transfer direction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Memory-to-memory
    MemoryToMemory,
    /// Memory-to-peripheral
    MemoryToPeripheral,
    /// Peripheral-to-memory
    PeripheralToMemory,
}

/// DMA transfer
// TODO - handle different word sizes
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Transfer<'d, T: Instance> {
    /// DMA channel
    pub _channel: &'d Channel<'d, T>, // TODO
}

impl<'d, T: Instance> Transfer<'d, T> {
    /// Reads from a peripheral register into a memory buffer using DMA
    pub fn new_read(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        peri_addr: *const u8, // TODO
        buf: &'d mut [u8],    // TODO
        options: &TransferOptions,
    ) -> Self {
        Self::new_read_raw(channel, request, peri_addr, buf, options)
    }

    /// Reads from a peripheral register into a memory buffer using DMA (raw pointers)
    pub fn new_read_raw(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        peri_addr: *const u8,
        buf: *mut [u8],
        options: &TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel,
            request,
            Direction::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut u8 as *mut u32, // TODO
            buf.len(),
            options,
        )
    }

    /// Writes a memory buffer into a peripheral register using DMA
    pub fn new_write(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        buf: &'d [u8],
        peri_addr: *mut u8,
        options: &TransferOptions,
    ) -> Self {
        Self::new_write_raw(channel, request, buf, peri_addr, options)
    }

    /// Writes a memory buffer into a peripheral register using DMA (raw pointers)
    pub fn new_write_raw(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        buf: *const [u8],   // TODO
        peri_addr: *mut u8, // TODO
        options: &TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel,
            request,
            Direction::MemoryToPeripheral,
            peri_addr as *mut u32,
            buf as *const u8 as *mut u32, // TODO
            buf.len(),
            options,
        )
    }

    /// Writes a memory buffer into another memory buffer using DMA
    pub fn new_write_mem(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        src_buf: &'d [u8],
        dst_buf: &'d mut [u8],
        options: &TransferOptions,
    ) -> Self {
        Self::new_write_mem_raw(channel, request, src_buf, dst_buf, options)
    }

    /// Writes a memory buffer into another memory buffer using DMA (raw pointers)
    pub fn new_write_mem_raw(
        channel: &'d mut Channel<'d, T>,
        request: Request,
        src_buf: *const [u8], // TODO
        dst_buf: *mut [u8],   // TODO
        options: &TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel,
            request,
            Direction::MemoryToMemory,
            src_buf as *const u32, // TODO
            dst_buf as *mut u32,
            src_buf.len(),
            options,
        )
    }

    /// Configures the channel for the read/write DMA transfer
    fn new_inner(
        channel: &'d mut Channel<'d, T>,
        _request: Request,
        dir: Direction,
        src_buf: *const u32,
        dst_buf: *mut u32,
        mem_len: usize,
        options: &TransferOptions,
    ) -> Self {
        // Configure the DMA channel descriptor and registers
        match channel.configure_channel(dir, src_buf, dst_buf, mem_len, options) {
            Ok(v) => v,
            Err(_e) => info!("failed to configure DMA channel number",),
        };
        // Enable the channel
        match channel.enable_channel() {
            Ok(v) => v,
            Err(_e) => info!("failed to enable DMA channel number",),
        };

        // Generate a software channel trigger to start the transfer
        match channel.trigger_channel() {
            Ok(v) => v,
            Err(_e) => info!("failed to trigger DMA channel number",),
        };

        Self { _channel: channel }
    }
}
