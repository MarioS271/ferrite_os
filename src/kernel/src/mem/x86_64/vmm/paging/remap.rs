// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `remap_page`
//!
//! Authors: MarioS271

use crate::mem::pmm::FRAME_SIZE;
use crate::mem::vmm::{VmmError, VmmResult};
use crate::mem::x86_64::vmm::page_type::PageType;
use crate::mem::x86_64::vmm::paging::helpers::{advance_current_pagetable, is_huge_page, is_present, misaligned_address_panic};
use crate::types::addr::VirtAddr;
use x86_64::instructions::tlb;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame};

pub unsafe fn remap_page(
    page_ptr: VirtAddr,
    virt: VirtAddr,
    new_flags: PageTableFlags
) -> VmmResult {
    #[cfg(debug_assertions)]
    if virt.as_u64() % FRAME_SIZE != 0 {
        misaligned_address_panic(PageType::Normal);
    }

    let mut current_pagetable: *mut PageTable = page_ptr.as_mut_ptr::<PageTable>();

    // PML4
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer allocated by the PMM and
    //   initialized by setup_kernel_paging
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p4_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidRemap) };
    current_pagetable = advance_current_pagetable(entry);

    // PDPT
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p3_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidRemap) };
    if is_huge_page(entry) {
        remap_huge_page(entry, virt, new_flags);
        return Ok(())
    }
    current_pagetable = advance_current_pagetable(entry);

    // PD
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p2_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidRemap) };
    if is_huge_page(entry) {
        remap_huge_page(entry, virt, new_flags);
        return Ok(())
    }
    current_pagetable = advance_current_pagetable(entry);

    // PT
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p1_index()] };
    if !is_present(entry) { return Err(VmmError::InvalidRemap) };
    entry.set_frame(
        PhysFrame::containing_address(entry.frame().unwrap().start_address()),
        new_flags | PageTableFlags::PRESENT
    );
    tlb::flush(virt.as_x86_64());
    debug_log(virt, new_flags);

    Ok(())
}

/// Wrapper for checking and remapping a huge page to avoid code duplication
#[inline]
fn remap_huge_page(entry: &mut PageTableEntry, virt: VirtAddr, new_flags: PageTableFlags) {
    entry.set_addr(entry.addr(), new_flags | PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE);
    tlb::flush(virt.as_x86_64());
    debug_log(virt, new_flags);
}

/// Debug log the remapping of a page
#[inline]
#[allow(unused_variables)]
fn debug_log(virt: VirtAddr, flags: PageTableFlags) {
    #[cfg(feature = "vmm-debug-logging")]
    crate::kdebug!("[VMM] remap page at virt {virt:#x} to flags {flags:?}");
}
