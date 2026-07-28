// SPDX-License-Identifier: GPL-3.0-only
//! Shared helper functions for the VMM.
//!
//! Authors: MarioS271

use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::paging::PhysFrame;

/// Panic with [`PanicCode::InvalidPageOperation`] when `unmap_page` encounters
/// a level of the page table walk that is not present.
pub fn invalid_unmap_panic() -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        "Attempting to unmap a page without a PRESENT flag",
    );
}

/// Panic with [`PanicCode::OutOfMemory`] when the VMM cannot allocate a frame
/// from the PMM (called via `unwrap_or_else` on `pmm::alloc()`).
pub fn out_of_memory_panic() -> ! {
    kernel_panic(
        PanicCode::OutOfMemory,
        "VMM could not allocate a frame, out of memory",
    )
}

/// Allocate one physical frame from the PMM, zero it, and return it as a [`PhysFrame`].
/// Used when `map_page` needs a new intermediate page-table frame.
///
/// # Panics
/// Panics if the PMM is out of memory.
pub fn alloc_zeroed_frame() -> PhysFrame {
    use crate::mem::pmm;
    use crate::mem::vmm::get;
    use x86_64::PhysAddr;
    use x86_64::structures::paging::PageTable;

    let frame = pmm::alloc().unwrap_or_else(|| out_of_memory_panic());

    // Safe because the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes((frame.as_u64() + get().hhdm_offset) as *mut PageTable, 0x00, 1);
    }

    PhysFrame::from_start_address(PhysAddr::new(frame.as_u64())).unwrap()
}