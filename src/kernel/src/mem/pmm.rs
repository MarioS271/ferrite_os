// SPDX-License-Identifier: GPL-3.0-only
//! Physical Memory Manager (PMM): tracks free and used 4 KiB physical frames with
//! a bitmap and hands out or reclaims individual frames.
//!
//! Authors: MarioS271

use crate::{kdebug, kemerg, kinfo};
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::PhysAddr;
use limine::memmap;
use limine::memmap::MEMMAP_USABLE;

pub static FRAME_SIZE: u64 = 4096;

/// Bitmap-based tracker of free and used physical frames.
pub struct Pmm {
    bitmap_ptr: *mut u8,
    total_frames: u64,
    bitmap_start_frame: u64,
    bitmap_end_frame: u64,
}

// TODO: write justification for this
unsafe impl Send for Pmm {}
unsafe impl Sync for Pmm {}

impl Pmm {
    /// Initialize the PMM from the Limine memory map.
    ///
    /// # Panics
    /// Panics if no single usable memory region is large enough to hold the bitmap.
    pub fn init(entries: &[&memmap::Entry], hhdm_offset: u64) -> Self {
        let mut max_entry = 0;

        for entry in entries {
            if entry.type_ != MEMMAP_USABLE {
                continue;
            }

            if entry.base + entry.length > max_entry {
                max_entry = entry.base + entry.length;
            }
        }

        let total_frames = max_entry / FRAME_SIZE;
        let bitmap_bytes = total_frames.div_ceil(8);

        kdebug!("[PMM] total_frames={total_frames}");
        kdebug!("[PMM] bitmap_bytes={bitmap_bytes}");

        let mut _bitmap_physical_base_addr: Option<u64> = None;
        let mut _bitmap_base_addr: Option<*mut u8> = None;

        for entry in entries {
            if entry.type_ != MEMMAP_USABLE {
                continue;
            }

            if entry.length >= bitmap_bytes {
                _bitmap_physical_base_addr = Some(entry.base);
                _bitmap_base_addr = Some((entry.base + hhdm_offset) as *mut u8);
                break;
            }
        }

        if _bitmap_physical_base_addr.is_none()
            || _bitmap_base_addr.is_none()
        {
            kernel_panic(
                PanicCode::NoValidMemMapEntry,
                "Could not find a usable memmap entry to place PMM bitmap in",
            );
        }

        let bitmap_physical_base_addr = _bitmap_physical_base_addr.unwrap();
        let bitmap_base_addr = _bitmap_base_addr.unwrap();

        kdebug!("[PMM] bitmap_physical_base_addr={bitmap_physical_base_addr}");
        kdebug!("[PMM] bitmap_base_addr={bitmap_base_addr:p}");

        // This is safe because we're writing over our bitmap which was determined from safe
        // limine-provided values.
        unsafe {
            core::ptr::write_bytes(bitmap_base_addr, 0xFF, bitmap_bytes as usize);
        }

        let bitmap_start_frame = bitmap_physical_base_addr / FRAME_SIZE;

        // Subtracting one so that the value is the last used frame, not the one after
        let bitmap_end_frame = ((bitmap_physical_base_addr + bitmap_bytes).div_ceil(FRAME_SIZE)) - 1;

        kdebug!("[PMM] bitmap_start_frame={bitmap_start_frame}");
        kdebug!("[PMM] bitmap_end_frame={bitmap_end_frame}");

        for entry in entries {
            if entry.type_ != MEMMAP_USABLE {
                continue;
            }

            let first_frame = entry.base / FRAME_SIZE;
            let frame_count = entry.length / FRAME_SIZE;

            for frame in first_frame..(first_frame + frame_count) {
                let frame_byte_offset = frame as usize / 8;
                let frame_bit_offset = frame % 8;

                if frame == 0 || (frame >= bitmap_start_frame && frame <= bitmap_end_frame) {
                    continue;
                }

                // Safe because we're iterating inside our bitmap with bound computed from the
                // limine values
                unsafe {
                    *bitmap_base_addr.add(frame_byte_offset) &= !(1 << (frame_bit_offset));
                }
            }
        }

        kinfo!("Initialized PMM");

        Pmm {
            bitmap_ptr: bitmap_base_addr,
            total_frames: total_frames,
            bitmap_start_frame: bitmap_start_frame,
            bitmap_end_frame: bitmap_end_frame,
        }
    }

    /// Allocate one free physical frame and return its address, or `None` if out of memory.
    pub fn alloc(&self) -> Option<PhysAddr> {
        for byte_index in 0..self.total_frames.div_ceil(8) {
            // Safe because we're operating inside the bitmap address range
            unsafe {
                let byte = *self.bitmap_ptr.add(byte_index as usize);

                if byte == 0xFF {
                    continue;
                }

                let bit_position = u8::trailing_ones(byte);
                *self.bitmap_ptr.add(byte_index as usize) |= 1 << bit_position;

                let frame_index = byte_index * 8 + bit_position as u64;
                let frame_address = frame_index * FRAME_SIZE;

                return Some(PhysAddr::new_truncate(frame_address));
            }
        }

        kemerg!("[PMM] unable to alloc() a frame, out of memory");

        None
    }

    /// Free a previously allocated physical frame.
    ///
    /// # Panics
    /// Panics if `addr` is frame 0, lies within the bitmap's own frames, is beyond
    /// tracked memory, or is already free (double-free).
    pub fn free(&self, addr: PhysAddr) {
        let frame_index = addr.as_u64() / FRAME_SIZE;

        if frame_index == 0 {
            kernel_panic(
                PanicCode::IllegalFree,
                "Attempting to Pmm::free() frame 0",
            );
        }

        if frame_index >= self.bitmap_start_frame && frame_index <= self.bitmap_end_frame {
            kernel_panic(
                PanicCode::IllegalFree,
                "Attempting to Pmm::free() in the range of the pmm bitmap",
            );
        }

        if frame_index >= self.total_frames {
            kernel_panic(
                PanicCode::IllegalFree,
                "Attempting to Pmm::free() outside of usable memory",
            );
        }

        let byte_offset = frame_index / 8;
        let bit_offset = frame_index % 8;

        // Safe because we're operating inside the bitmap address range
        unsafe {
            let byte = *self.bitmap_ptr.add(byte_offset as usize);

            if byte & (1 << bit_offset) == 0u8 {
                kernel_panic(
                    PanicCode::DoubleFree,
                    "Attempting to Pmm::free() a frame which is already freed",
                );
            }

            *self.bitmap_ptr.add(byte_offset as usize) &= !(1 << bit_offset);
        }
    }
}
