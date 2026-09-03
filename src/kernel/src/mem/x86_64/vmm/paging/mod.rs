// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): mapping, unmapping, remapping pages and more
//!
//! Authors: MarioS271

mod helpers;
mod setup_kernel_paging;
mod map;
mod unmap;
mod remap;
mod translate;
mod vma_flags_to_page_flags;

use crate::mem::pmm::Pmm;
use crate::mem::vmm::traits::VmmPaging;
use crate::mem::vmm::vma::VmaFlags;
use crate::mem::vmm::{Vmm, VmmResult};
use crate::mem::x86_64::vmm::page_type::PageType;
use crate::types::addr::{PhysAddr, VirtAddr};
use x86_64::structures::paging::PageTableFlags;

impl VmmPaging for Vmm {
    type PageType = PageType;
    type PageTableFlags = PageTableFlags;

    #[inline]
    fn setup_kernel_paging() -> VirtAddr {
        setup_kernel_paging::setup_kernel_paging()
    }

    #[inline]
    unsafe fn map_page(
        pmm: &mut Pmm,
        page_ptr: VirtAddr,
        virt: VirtAddr,
        phys: PhysAddr,
        page_type: Self::PageType,
        flags: Self::PageTableFlags
    ) -> VmmResult {
        unsafe { map::map_page(pmm, page_ptr, virt, phys, page_type, flags) }
    }

    #[inline]
    unsafe fn unmap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> VmmResult {
        unsafe { unmap::unmap_page(page_ptr, virt) }
    }

    #[inline]
    unsafe fn remap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr,
        new_flags: Self::PageTableFlags
    ) -> VmmResult {
        unsafe { remap::remap_page(page_ptr, virt, new_flags) }
    }

    #[inline]
    fn translate(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> Option<PhysAddr> {
        unsafe { translate::translate(page_ptr, virt) }
    }

    #[inline]
    fn translate_with_size(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> Option<(PhysAddr, u64)> {
        unsafe { translate::translate_with_size(page_ptr, virt) }
    }

    #[inline]
    fn vma_flags_to_page_flags(
        vma_flags: VmaFlags
    ) -> Self::PageTableFlags {
        vma_flags_to_page_flags::vma_flags_to_page_flags(vma_flags)
    }
}
