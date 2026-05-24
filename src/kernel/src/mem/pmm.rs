//! mem/pmm.rs
//! Physical Memory Manager
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

static FRAME_SIZE: u64 = 4096;     // 4 KiB frame size
static PMM_DATA: Once<PmmData> = Once::new();

struct PmmData {
    bitmap_ptr: *mut u8,
    total_frames: u64,
    bitmap_start_frame: u64,
    bitmap_end_frame: u64,
}

// Safe because all access is controlled via the Once
unsafe impl Send for PmmData {}
unsafe impl Sync for PmmData {}

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
            true
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
    let bitmap_end_frame = (bitmap_physical_base_addr + bitmap_bytes).div_ceil(FRAME_SIZE);

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

            if frame == 0 || (frame >= bitmap_start_frame && frame < bitmap_end_frame) {
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

pub fn alloc() -> Option<PhysAddr> {
    if PMM_DATA.get().is_none() {
        kernel_panic(
            PanicCode::PmmNotInitialized,
            "Cannot alloc(), PMM is not initialized",
            true
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
            *pmm.bitmap_ptr.add(byte_index as usize) |= (1 << bit_position);

            let frame_index = byte_index * 8 + bit_position as u64;
            let frame_address = frame_index * FRAME_SIZE;

            kprint!("[PMM] allocated frame {frame_index} at addr {frame_address}\n");

            return Some(PhysAddr::new_truncate(frame_address));
        }
    }

    kprint!("[PMM] unable to alloc() a frame, out of memory");

    None
}

pub fn free(addr: PhysAddr) {
    if PMM_DATA.get().is_none() {
        kernel_panic(
            PanicCode::PmmNotInitialized,
            "Cannot free(), PMM is not initialized",
            true
        );
    }

    let pmm = PMM_DATA.get().unwrap();
    let frame_index = addr.as_u64() / FRAME_SIZE;

    if frame_index == 0 {
        kernel_panic(
            PanicCode::IllegalFree,
            "Attempting to free() frame 0",
            true
        );
    }

    if frame_index >= pmm.bitmap_start_frame && frame_index < pmm.bitmap_end_frame {
        kernel_panic(
            PanicCode::IllegalFree,
            "Attempting to free() in the range of the pmm bitmap",
            true
        );
    }

    if frame_index >= pmm.total_frames {
        kernel_panic(
            PanicCode::IllegalFree,
            "Attempting to free() outside of usable memory",
            true
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
                "Attempting to free() a frame which is already freed",
                true
            );
        }

        *pmm.bitmap_ptr.add(byte_offset as usize) &= !(1 << bit_offset);
    }
}