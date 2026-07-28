// SPDX-License-Identifier: GPL-3.0-only
//! Shared helper functions for the VMM.
//!
//! Authors: MarioS271

use crate::mem::pmm::Pmm;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::paging::{PageTable, PhysFrame};
use x86_64::PhysAddr;

/// Panic when `unmap_page` hits a page-table level that is not present.
pub fn invalid_unmap_panic() -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        "Attempting to unmap a page without a PRESENT flag",
    );
}

/// Panic when the VMM cannot allocate a frame from the PMM.
pub fn out_of_memory_panic() -> ! {
    kernel_panic(
        PanicCode::OutOfMemory,
        "VMM could not allocate a frame, out of memory",
    )
}

/// Allocate one physical frame from the PMM, zero it, and return it as a [`PhysFrame`].
///
/// # Panics
/// Panics if the PMM is out of memory.
pub fn alloc_zeroed_frame(pmm: &Pmm, hhdm_offset: u64) -> PhysFrame {
    let frame = pmm.alloc().unwrap_or_else(|| out_of_memory_panic());

    // Safe because the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes((frame.as_u64() + hhdm_offset) as *mut PageTable, 0x00, 1);
    }

    PhysFrame::from_start_address(PhysAddr::new(frame.as_u64())).unwrap()
}
