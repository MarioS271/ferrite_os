//! mem/x86_64/vmm_helpers.rs
//! Virtual Memory Manager Helper Functions
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::paging::PhysFrame;

pub fn invalid_unmap_panic() -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        "Attempting to unmap a page without a PRESENT flag",
        true
    );
}

pub fn out_of_memory_panic() -> ! {
    kernel_panic(
        PanicCode::OutOfMemory,
        "VMM could not allocate a frame, out of memory",
        true
    )
}

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