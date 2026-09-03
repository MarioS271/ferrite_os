// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `vma_flags_to_page_flags`
//!
//! Authors: MarioS271

use crate::mem::vmm::vma::VmaFlags;
use x86_64::structures::paging::PageTableFlags;

pub fn vma_flags_to_page_flags(
    vma_flags: VmaFlags
) -> PageTableFlags {
    let mut flags = PageTableFlags::PRESENT;

    if vma_flags.contains(VmaFlags::WRITE) {
        flags |= PageTableFlags::WRITABLE;
    }
    if !vma_flags.contains(VmaFlags::EXEC) {
        flags |= PageTableFlags::NO_EXECUTE;
    }
    if vma_flags.contains(VmaFlags::USER) {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    } else {
        flags |= PageTableFlags::GLOBAL;
    }

    flags
}