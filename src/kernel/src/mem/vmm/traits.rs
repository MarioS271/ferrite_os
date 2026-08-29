// SPDX-License-Identifier: GPL-3.0-only
//!
//!
//! Authors: MarioS271

use crate::mem::pmm::Pmm;
use crate::mem::vmm::vma::VmaFlags;
use crate::types::addr::{PhysAddr, VirtAddr};

pub trait VmmPaging {
    type PageType;
    type PageTableFlags;

    /// Initialize the kernel page tables and load them
    ///
    /// # Panics
    /// Panics if the PMM cannot allocate memory for the page tables
    fn setup_kernel_page() -> VirtAddr;

    /// Map one 4 KiB virtual page to a physical frame. `PRESENT` is forced on regardless of `flags`.
    ///
    /// # Safety
    /// The caller must ensure `virt` is a valid kernel virtual address and `phys` is a
    /// valid, PMM-allocated frame.
    ///
    /// # Panics
    /// Panics if the PMM runs out of frames when allocating an intermediate page table.
    unsafe fn map_page(
        pmm: &mut Pmm,
        page_ptr: VirtAddr,
        virt: VirtAddr,
        phys: PhysAddr,
        page_type: Self::PageType,
        flags: Self::PageTableFlags
    );

    /// Unmap one virtual page
    ///
    /// # Safety
    /// The caller must ensure `virt` was previously mapped and that no live code or
    /// data references the page after this call returns.
    ///
    /// # Panics
    /// Panics if any level of the walk is not present (page was never mapped).
    unsafe fn unmap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr
    );

    /// Change page table flags of an already mapped page
    ///
    /// # Safety
    /// The caller must ensure `virt` was previously mapped and that changing the page's flags
    /// will not violate anything (example: making a page non-writable while a mutable reference
    /// is held to it)
    ///
    /// # Panics
    /// Panics if any level of the walk is not present (page was never mapped).
    unsafe fn remap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr,
        new_flags: Self::PageTableFlags
    );

    /// Walk the page table and return the phys address mapped at `virt` or `None` if any
    /// level is not present
    fn translate(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> Option<PhysAddr>;

    /// Translate VMA flags to arch-specific page table flags
    fn vma_flags_to_page_flags(
        vma_flags: VmaFlags
    ) -> Self::PageTableFlags;

    /// Translates a page type (like HugePage(2MiB) on x86_64) to an u64 size
    fn page_type_to_size(
        page_type: Self::PageType
    ) -> u64;
}
