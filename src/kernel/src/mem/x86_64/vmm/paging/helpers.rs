// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64) helpers
//!
//! Authors: MarioS271

use crate::mem::x86_64::vmm::page_type::PageType;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{PageTable, PageTableFlags};

/// Panic when a given address is not aligned properly
pub fn misaligned_address_panic(align: PageType) -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        match align {
            PageType::Normal => "Physical and/or virtual address is not aligned to 4 KiB",
            PageType::HugePage2MiB => "Physical and/or virtual address is not aligned to 2 MiB",
            PageType::HugePage1GiB => "Physical and/or virtual address is not aligned to 1 GiB"
        }
    );
}

/// Check whether the given [`PageTableEntry`] has the `PRESENT` property
#[inline]
pub fn is_present(entry: &PageTableEntry) -> bool {
    entry.flags().contains(PageTableFlags::PRESENT)
}

/// Check whether the given [`PageTableEntry`] has the `HUGE_PAGE` property
#[inline]
pub fn is_huge_page(entry: &PageTableEntry) -> bool {
    entry.flags().contains(PageTableFlags::HUGE_PAGE)
}

/// Returns a mutable pointer to the next page table
#[inline]
pub fn advance_current_pagetable(entry: &PageTableEntry) -> *mut PageTable {
    (entry.frame().unwrap().start_address().as_u64() + KSTATE.mm.hhdm_offset()) as *mut PageTable
}
