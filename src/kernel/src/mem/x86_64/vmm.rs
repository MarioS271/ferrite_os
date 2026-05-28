//! mem/vmm.rs
//! Virtual Memory Manager
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::mem::pmm;
use crate::mem::pmm::FRAME_SIZE;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PageTable;

struct VmmData {

}

pub fn init(hhdm_offset: u64) {
    let limine_plm4_ptr = (Cr3::read().0.start_address().as_u64() + hhdm_offset) as *const PageTable;

    let plm4_ptr = (pmm::alloc().unwrap_or_else(
        || kernel_panic(
            PanicCode::OutOfMemory,
            "VMM could not allocate PLM4 page, out of memory",
            true
        )
    ).as_u64() + hhdm_offset) as *mut u8;

    // Safe because the PMM gives us a valid piece of memory with a size of pmm::FRAME_SIZE
    unsafe {
        core::ptr::write_bytes(plm4_ptr, 0x00, FRAME_SIZE as usize);
    }


}