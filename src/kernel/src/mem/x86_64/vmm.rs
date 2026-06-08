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
use x86_64::structures::paging::{PageTable, PhysFrame};
use x86_64::PhysAddr;

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
    ).as_u64() + hhdm_offset) as *mut PageTable;

    // Safe because the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes(plm4_ptr, 0x00, 1);
    }

    for entry in 256..512 {
        // Both pointers are valid and only touched here at time of execution
        unsafe {
            (&mut (*plm4_ptr))[entry] = (&(*limine_plm4_ptr))[entry].clone();
        }
    }

    // Not using .sub() here as it subtracts in units of the pointed to thing (which would
    // be PageTable)
    let phys_addr_u64 = plm4_ptr as u64 - hhdm_offset;
    let phys_frame = PhysFrame::containing_address(PhysAddr::new(phys_addr_u64));
    let current_cr3_flags = Cr3::read().1;

    // Using the same plm4 as provided by limine before, just copied so that the kernel
    // is able to own it (limine's plm4 was safe and functional)
    unsafe {
        Cr3::write(phys_frame, current_cr3_flags);
    }
}