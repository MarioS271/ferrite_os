// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `unmap_page`
//!
//! Authors: MarioS271

use super::helpers::{advance_current_pagetable, is_huge_page, is_present};
use crate::lib::addr::VirtAddr;
use crate::mm::vmm::{VmmError, VmmResult};
use x86_64::instructions::tlb;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::PageTable;

pub unsafe fn unmap_page(
    page_ptr: VirtAddr,
    virt: VirtAddr
) -> VmmResult {
    #[cfg(feature = "debug-checks")]
    {
        use crate::mm::pmm::FRAME_SIZE;
        use super::super::page_type::PageType;
        use super::helpers::misaligned_address_panic;

        if virt.as_u64() % FRAME_SIZE != 0 {
            misaligned_address_panic(PageType::Normal);
        }
    }

    let mut current_pagetable = page_ptr.as_mut_ptr::<PageTable>();

    // PML4
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer allocated by the PMM and
    //   initialized by setup_kernel_paging
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p4_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidUnmap) };
    current_pagetable = advance_current_pagetable(entry);

    // PDPT
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p3_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidUnmap) };
    if is_huge_page(entry) {
        clear_and_flush(entry, virt);
        debug_log(virt);
        return Ok(());
    }
    current_pagetable = advance_current_pagetable(entry);

    // PD
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p2_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidUnmap) };
    if is_huge_page(entry) {
        clear_and_flush(entry, virt);
        debug_log(virt);
        return Ok(());
    }
    current_pagetable = advance_current_pagetable(entry);

    // PT
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p1_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidUnmap) };
    clear_and_flush(entry, virt);
    debug_log(virt);

    Ok(())
}

/// Clears the given [`PageTableEntry`] and flushes the given [`VirtAddr`]
fn clear_and_flush(entry: &mut PageTableEntry, virt: VirtAddr) {
    entry.set_unused();
    tlb::flush(virt.as_x86_64());
}

/// Debug log the unmapping of a page
#[allow(unused_variables)]
fn debug_log(virt: VirtAddr) {
    #[cfg(feature = "vmm-debug-logging")]
    crate::kdebug!("[VMM] unmap page at virt {virt:#x}");
}
