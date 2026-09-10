// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): mapping, unmapping, remapping pages and more
//!
//! Authors: MarioS271

mod helpers;
mod setup_kernel_paging;
mod map_page;
mod unmap;
mod remap;
mod translate;
mod map_range;

use super::page_type::PageType;
use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::lib::types::boot_info::KernelSectionInfo;
use crate::mm::pmm::Pmm;
use crate::mm::vmm::boot_mapping::BootMappings;
use crate::mm::vmm::traits::VmmPaging;
use crate::mm::vmm::vma::VmaFlags;
use crate::mm::vmm::{Vmm, VmmResult};
use crate::state::kstate::KSTATE;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::PageTableFlags;

impl VmmPaging for Vmm {
    type PageType = PageType;
    type PageTableFlags = PageTableFlags;

    #[inline(always)]
    fn setup_kernel_paging(sections: &KernelSectionInfo) -> (VirtAddr, BootMappings) {
        setup_kernel_paging::setup_kernel_paging(sections)
    }

    #[inline(always)]
    unsafe fn map_page(
        pmm: &mut Pmm,
        page_ptr: VirtAddr,
        virt: VirtAddr,
        phys: PhysAddr,
        page_type: Self::PageType,
        flags: Self::PageTableFlags
    ) -> VmmResult {
        unsafe { map_page::map_page(pmm, page_ptr, virt, phys, page_type, flags) }
    }

    #[inline(always)]
    unsafe fn map_range(
        pmm: &mut Pmm,
        page_ptr: VirtAddr,
        virt: VirtAddr,
        phys: PhysAddr,
        size: u64,
        flags: Self::PageTableFlags,
        can_overmap: bool
    ) -> VmmResult {
        unsafe { map_range::map_range(pmm, page_ptr, virt, phys, size, flags, can_overmap) }
    }

    #[inline(always)]
    unsafe fn unmap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> VmmResult {
        unsafe { unmap::unmap_page(page_ptr, virt) }
    }

    #[inline(always)]
    unsafe fn remap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr,
        new_flags: Self::PageTableFlags
    ) -> VmmResult {
        unsafe { remap::remap_page(page_ptr, virt, new_flags) }
    }

    #[inline(always)]
    fn translate(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> Option<PhysAddr> {
        unsafe { translate::translate(page_ptr, virt) }
    }

    #[inline(always)]
    fn translate_with_size(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> Option<(PhysAddr, u64)> {
        unsafe { translate::translate_with_size(page_ptr, virt) }
    }

    fn vma_flags_to_page_flags(vma_flags: VmaFlags) -> Self::PageTableFlags {
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

    unsafe fn clone_kernel_mappings(dst: VirtAddr) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                KSTATE.mm.kernel_addr_space().lock().page_ptr().as_ptr::<PageTableEntry>().add(256),
                dst.as_mut_ptr::<PageTableEntry>().add(256),
                256
            );
        }
    }
}
