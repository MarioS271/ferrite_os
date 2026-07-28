//! Physical Memory Manager (PMM).
//!
//! Tracks which 4 KiB physical frames are free using a flat bitmap: one bit per
//! frame, `0` = free, `1` = used. The bitmap itself is placed inside the first
//! usable memory region large enough to hold it, and those frames are permanently
//! marked as used so they are never handed out.
//!
//! The public API is intentionally minimal: [`init`] (called once from `kmain`),
//! [`alloc`] (returns the physical address of one free frame), and [`free`]
//! (marks a previously allocated frame as free again). All three functions panic
//! on detected misuse rather than returning errors, since a memory manager
//! operating on bad inputs cannot be corrected at runtime.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::kprint;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use spin::Once;
use x86_64::PhysAddr;
use limine::memmap;
use limine::memmap::MEMMAP_USABLE;

/// The granularity of all physical memory operations. Every allocation and free
/// operates on exactly one frame of this size.
pub static FRAME_SIZE: u64 = 4096;

/// Global PMM state, initialized exactly once by [`init`].
static PMM_DATA: Once<PmmData> = Once::new();

/// Internal PMM state: the bitmap pointer and the frame index range it occupies.
struct PmmData {
    /// Pointer to the first byte of the allocation bitmap, in virtual address space
    /// (physical base + HHDM offset). Each bit represents one physical frame.
    bitmap_ptr: *mut u8,
    /// Total number of frames covered by the bitmap (derived from the highest
    /// physical address in any usable memmap entry).
    total_frames: u64,
    /// Index of the first frame occupied by the bitmap itself. Frames in
    /// `bitmap_start_frame..=bitmap_end_frame` are permanently reserved.
    bitmap_start_frame: u64,
    /// Index of the last frame occupied by the bitmap itself.
    bitmap_end_frame: u64,
}

// Safe because all access to PmmData fields goes through the Once, and the
// kernel is currently single-threaded (no concurrent mutation).
unsafe impl Send for PmmData {}
unsafe impl Sync for PmmData {}

/// Initialize the PMM from the Limine memory map.
///
/// Scans `entries` for the highest usable physical address to determine how many
/// frames exist, then computes how many bytes the bitmap needs. Finds the first
/// usable region large enough to hold the bitmap and places it there. Initializes
/// all bitmap bits to 1 (all used), then clears bits for each usable frame that is
/// not frame 0 and not inside the bitmap itself.
///
/// # Panics
/// Panics if no single usable memory region is large enough to hold the bitmap.
pub fn init(entries: &[&memmap::Entry], hhdm_offset: u64) {
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

    kprint!("[PMM] total_frames={total_frames}\n");
    kprint!("[PMM] bitmap_bytes={bitmap_bytes}\n");

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

    kprint!("[PMM] bitmap_physical_base_addr={bitmap_physical_base_addr}\n");
    kprint!("[PMM] bitmap_base_addr={bitmap_base_addr:p}\n");

    // This is safe because we're writing over our bitmap which was determined from safe
    // limine-provided values.
    unsafe {
        core::ptr::write_bytes(bitmap_base_addr, 0xFF, bitmap_bytes as usize);
    }

    let bitmap_start_frame = bitmap_physical_base_addr / FRAME_SIZE;

    // Subtracting one so that the value is the last used frame, not the one after
    let bitmap_end_frame = ((bitmap_physical_base_addr + bitmap_bytes).div_ceil(FRAME_SIZE)) - 1;

    kprint!("[PMM] bitmap_start_frame={bitmap_start_frame}\n");
    kprint!("[PMM] bitmap_end_frame={bitmap_end_frame}\n");

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

    PMM_DATA.call_once(|| PmmData{
        bitmap_ptr: bitmap_base_addr,
        total_frames: total_frames,
        bitmap_start_frame: bitmap_start_frame,
        bitmap_end_frame: bitmap_end_frame,
    });
}

/// Allocate one free physical frame and return its address.
///
/// Scans the bitmap from index 0 for the first byte that is not `0xFF` (fully
/// used), finds the lowest zero bit within that byte, marks it as used, and
/// returns the corresponding physical address. Returns `None` if all frames are
/// used; also prints a warning via `kprint!`.
///
/// # Panics
/// Panics if called before [`init`].
pub fn alloc() -> Option<PhysAddr> {
    if PMM_DATA.get().is_none() {
        kernel_panic(
            PanicCode::PmmNotInitialized,
            "Cannot pmm::alloc(), PMM is not initialized",
        );
    }

    let pmm = PMM_DATA.get().unwrap();

    for byte_index in 0..pmm.total_frames.div_ceil(8) {
        // Safe because we're operating inside the bitmap address range
        unsafe {
            let byte = *pmm.bitmap_ptr.add(byte_index as usize);

            if byte == 0xFF {
                continue;
            }

            let bit_position = u8::trailing_ones(byte);
            *pmm.bitmap_ptr.add(byte_index as usize) |= 1 << bit_position;

            let frame_index = byte_index * 8 + bit_position as u64;
            let frame_address = frame_index * FRAME_SIZE;

            return Some(PhysAddr::new_truncate(frame_address));
        }
    }

    kprint!("[PMM] unable to alloc() a frame, out of memory");

    None
}

/// Free a previously allocated physical frame.
///
/// Computes the frame index from `addr`, validates it (not frame 0, not inside
/// the bitmap, not beyond `total_frames`, and not already free), then clears the
/// corresponding bitmap bit.
///
/// # Panics
/// Panics via [`kernel_panic`] on any of the following:
/// - PMM not yet initialized.
/// - `addr` points to frame 0 (the null frame is never allocatable).
/// - `addr` falls within the range occupied by the PMM bitmap.
/// - `addr` is beyond the tracked address space.
/// - The frame is already marked free (double-free detection).
pub fn free(addr: PhysAddr) {
    if PMM_DATA.get().is_none() {
        kernel_panic(
            PanicCode::PmmNotInitialized,
            "Cannot pmm::free(), PMM is not initialized",
        );
    }

    let pmm = PMM_DATA.get().unwrap();
    let frame_index = addr.as_u64() / FRAME_SIZE;

    if frame_index == 0 {
        kernel_panic(
            PanicCode::IllegalFree,
            "Attempting to pmm::free() frame 0",
        );
    }

    if frame_index >= pmm.bitmap_start_frame && frame_index <= pmm.bitmap_end_frame {
        kernel_panic(
            PanicCode::IllegalFree,
            "Attempting to pmm::free() in the range of the pmm bitmap",
        );
    }

    if frame_index >= pmm.total_frames {
        kernel_panic(
            PanicCode::IllegalFree,
            "Attempting to pmm::free() outside of usable memory",
        );
    }

    let byte_offset = frame_index / 8;
    let bit_offset = frame_index % 8;

    // Safe because we're operating inside the bitmap address range
    unsafe {
        let byte = *pmm.bitmap_ptr.add(byte_offset as usize);

        if byte & (1 << bit_offset) == 0u8 {
            kernel_panic(
                PanicCode::DoubleFree,
                "Attempting to pmm::free() a frame which is already freed",
            );
        }

        *pmm.bitmap_ptr.add(byte_offset as usize) &= !(1 << bit_offset);
    }
}
