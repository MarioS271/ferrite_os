// SPDX-License-Identifier: GPL-3.0-only
//! Contains VMM definitions such as the VMM struct, the HugePageType enum and more
//!
//! Authors: MarioS271

use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, PhysFrame};
use x86_64::structures::paging::page_table::PageTableEntry;
use crate::kinfo;
use crate::mem::pmm::Pmm;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::addr::{PhysAddr, VirtAddr};
use crate::types::panic_codes::PanicCode;

/// Kernel page-table state used by all VMM operations.
pub struct Vmm;

// Safe because Vmm is written once during init (single-threaded) and then
// only read after that.
unsafe impl Send for Vmm {}
unsafe impl Sync for Vmm {}

impl Vmm {
    /// Initialize the kernel PML4 and load it into CR3
    ///
    /// # Panics
    /// Panics if the PMM cannot allocate the PML4 frame (out of memory).
    pub fn setup_kernel_paging() -> VirtAddr {
        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut pmm = KSTATE.mm.pmm.get().unwrap().lock();

        let limine_pml4_ptr = (Cr3::read().0.start_address().as_u64() + hhdm_offset) as *const PageTable;

        let kernel_pml4_ptr = (
            pmm.alloc_frame().unwrap_or_else(|| out_of_memory_panic()).as_u64() + hhdm_offset
        ) as *mut PageTable;

        // Safe because the PMM gives us a valid piece of memory
        unsafe {
            core::ptr::write_bytes(kernel_pml4_ptr, 0x00, 1);
        }

        unsafe {
            core::ptr::copy_nonoverlapping(
                (limine_pml4_ptr as *const PageTableEntry).add(256),
                (kernel_pml4_ptr as *mut PageTableEntry).add(256),
                256
            );
        }

        let phys_addr_u64 = kernel_pml4_ptr as u64 - hhdm_offset;
        let phys_frame = PhysFrame::containing_address(x86_64::PhysAddr::new(phys_addr_u64));
        let current_cr3_flags = Cr3::read().1;

        // Using the same PML4 as provided by limine before, just copied so that the kernel
        // is able to own it (limine's plm4 was safe and functional)
        unsafe {
            Cr3::write(phys_frame, current_cr3_flags);
        }

        kinfo!("Initialized kernel PML4 (Phys Addr: {phys_addr_u64:#x})");

        VirtAddr::new(kernel_pml4_ptr as u64)
    }
}

/// Allocate one physical frame from the PMM, zero it, and return it as a [`PhysFrame`].
///
/// # Panics
/// Panics if the PMM is out of memory.
pub(super) fn alloc_zeroed_frame(pmm: &mut Pmm) -> PhysFrame {
    let hhdm_offset = &KSTATE.mm.hhdm_offset();
    let frame = pmm.alloc_frame().unwrap_or_else(|| out_of_memory_panic());

    // Safe because the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes((frame.as_u64() + hhdm_offset) as *mut PageTable, 0x00, 1);
    }

    PhysFrame::from_start_address(x86_64::PhysAddr::new(frame.as_u64())).unwrap()
}

/// Panic when the VMM cannot allocate a frame from the PMM.
pub(super) fn out_of_memory_panic() -> ! {
    kernel_panic(
        PanicCode::OutOfMemory,
        "VMM could not allocate a frame, out of memory",
    )
}
