// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `translate` and `translate_with_size`
//!
//! Authors: MarioS271

use super::super::page_type::{HUGE_PAGE_SIZE_1GIB, HUGE_PAGE_SIZE_2MIB};
use super::helpers::{advance_current_pagetable, is_huge_page, is_present};
use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::mm::pmm::FRAME_SIZE;
use x86_64::structures::paging::PageTable;

pub unsafe fn translate(
    page_ptr: VirtAddr,
    virt: VirtAddr
) -> Option<PhysAddr> {
    let res = unsafe { translate_with_size(page_ptr, virt)? };
    Some(res.0)
}

pub unsafe fn translate_with_size(
    page_ptr: VirtAddr,
    virt: VirtAddr
) -> Option<(PhysAddr, u64)> {
    let mut current = page_ptr.as_mut_ptr::<PageTable>();

    // PML4
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer allocated by the PMM and
    //   initialized by setup_kernel_paging
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &(&*current)[virt.p4_index()] };
    if !is_present(entry) { return None; }
    current = advance_current_pagetable(entry);

    // PDPT
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &(&*current)[virt.p3_index()] };
    if !is_present(entry) { return None; }
    if is_huge_page(entry) {
        return Some((
            PhysAddr::new(entry.addr().as_u64() + (virt.as_u64() & (HUGE_PAGE_SIZE_1GIB - 1))),
            HUGE_PAGE_SIZE_1GIB
        ));
    }
    current = advance_current_pagetable(entry);

    // PD
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &(&*current)[virt.p2_index()] };
    if !is_present(entry) { return None; }
    if is_huge_page(entry) {
        return Some((
            PhysAddr::new(entry.addr().as_u64() + (virt.as_u64() & (HUGE_PAGE_SIZE_2MIB - 1))),
            HUGE_PAGE_SIZE_2MIB
        ));
    }
    current = advance_current_pagetable(entry);

    // PT
    // Safety:
    // - current_pagetable is a valid, non-zero, page-aligned pointer computed from the previous
    //   current_pagetable which was also valid
    // - the VMM is wrapped in an IrqMutex which guarantees serialized VMM method execution
    let entry = unsafe { &(&*current)[virt.p1_index()] };
    if !is_present(entry) { return None; }
    Some((
        PhysAddr::new(entry.addr().as_u64() + (virt.as_u64() & (FRAME_SIZE - 1))),
        FRAME_SIZE
    ))
}
