// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `setup_kernel_paging`
//!
//! Authors: MarioS271

use crate::kinfo;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::addr::VirtAddr;
use crate::types::panic_codes::PanicCode;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{PageTable, PhysFrame};

pub fn setup_kernel_paging() -> VirtAddr {
    // TODO: do own paging

    let hhdm_offset = &KSTATE.mm.hhdm_offset();
    let limine_page_ptr = (Cr3::read().0.start_address().as_u64() + hhdm_offset) as *const PageTable;

    // Safety: the PMM gets initialized before paging in mm_init
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

    let frame = pmm.alloc_frame().unwrap_or_else(
        || kernel_panic(
            PanicCode::OutOfMemory,
            "Could not allocate a PML4 for the kernel, out of memory"
        )
    );
    let kernel_page_ptr = frame.as_mut_hhdm_ptr::<PageTable>();

    // Safety: the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes(kernel_page_ptr, 0x00, 1);
    }

    unsafe {
        core::ptr::copy_nonoverlapping(
            (limine_page_ptr as *const PageTableEntry).add(256),
            (kernel_page_ptr as *mut PageTableEntry).add(256),
            256
        );
    }

    let phys_addr_u64 = kernel_page_ptr as u64 - hhdm_offset;
    let phys_frame = PhysFrame::containing_address(x86_64::PhysAddr::new(phys_addr_u64));
    let current_cr3_flags = Cr3::read().1;

    // Safety: Using the same PML4 as provided by limine before, just copied so that the kernel
    // is able to own it (limine's PML4 was safe and functional)
    unsafe {
        Cr3::write(phys_frame, current_cr3_flags);
    }

    kinfo!("Initialized kernel PML4 (Phys Addr: {phys_addr_u64:#x})");

    VirtAddr::new(kernel_page_ptr as u64)
}