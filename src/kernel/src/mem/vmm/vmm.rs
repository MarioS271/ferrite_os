// SPDX-License-Identifier: GPL-3.0-only
//! VMM definitions (VMM struct, common helpers)
//!
//! Authors: MarioS271

use crate::mem::pmm::{Pmm, FRAME_SIZE};
use crate::mem::vmm::address_space::AddressSpace;
use crate::mem::vmm::traits::VmmPaging;
use crate::mem::vmm::vma::{Vma, VmaFlags};
use crate::panic::kernel_panic;
use crate::types::addr::VirtAddr;
use crate::types::panic_codes::PanicCode;

/// Namespace for all VMM methods
pub struct Vmm;

/// Error type for VMM operations
pub type VmmResult = Result<(), VmmError>;

/// VMM Error Enum with VMM error types
pub enum VmmError {
    VmaNotFound,
    VmaOverlap
}

impl Vmm {
    /// Map a virtual memory region by creating a VMA in the given [`AddressSpace`]
    ///
    /// # Panics
    /// Panics in debug builds if either `virt` is not aligned to [`FRAME_SIZE`] or `size`
    /// is not a multiple of [`FRAME_SIZE`]
    ///
    /// # Returns
    /// Returns [`VmmError::VmaOverlap`] if the given region overlaps with an existing VMA
    pub fn map_region(
        address_space: &mut AddressSpace,
        virt: VirtAddr,
        size: u64,
        vma_flags: VmaFlags
    ) -> VmmResult {
        #[cfg(debug_assertions)]
        if !virt.is_aligned(FRAME_SIZE) || size % FRAME_SIZE != 0 {
            kernel_panic(
                PanicCode::MisalignedAddress,
                "Vmm::map_region recieved an improperly aligned virtual address or size"
            );
        }

        address_space.insert_vma(
            Vma {
                start_addr: virt,
                end_addr: virt + size,
                flags: vma_flags
            }
        )
    }

    /// Unmap a virtual memory region by removing its VMA from the given [`AddressSpace`] and
    /// unmapping all currently mapped pages
    ///
    /// # Safety
    /// The caller must ensure that no live code or data holds references into any address mapped
    /// within `[virt, vma.end_addr)` after this method call returns
    ///
    /// # Panics
    /// Panics in debug builds if `virt` is not aligned to [`FRAME_SIZE`]
    ///
    /// # Returns
    /// Returns [`VmmError::VmaNotFound`] if no VMA starting at `virt` exists
    pub unsafe fn unmap_region(
        pmm: &mut Pmm,
        address_space: &mut AddressSpace,
        virt: VirtAddr
    ) -> VmmResult
    where
        Self: VmmPaging
    {
        #[cfg(debug_assertions)]
        if !virt.is_aligned(FRAME_SIZE) {
            kernel_panic(
                PanicCode::MisalignedAddress,
                "Vmm::unmap_region recieved an improperly aligned virtual address or size"
            );
        }

        let vma = address_space.remove_vma(virt).ok_or(VmmError::VmaNotFound)?;
        let size = vma.end_addr.as_u64() - vma.start_addr.as_u64();

        let mut offset = 0;
        while offset < size {
            if let Some(phys) = Self::translate(address_space.page_ptr, virt + offset) {
                unsafe { Self::unmap_page(address_space.page_ptr, virt + offset); }
                pmm.free_frame(phys);
            }

            offset += FRAME_SIZE;
        }

        Ok(())
    }

    /// Remap a virtual memory region by changing its VMA's flags from the given [`AddressSpace`] and
    /// remapping all currently mapped pages to new matching page flags.
    ///
    /// # Safety
    /// The caller must ensure that for example when removing the `WRITE` flag,
    /// no mutable references to the to be modified memory is held.
    ///
    /// # Panics
    /// Panics in debug builds if `virt` is not aligned to [`FRAME_SIZE`]
    ///
    /// # Returns
    /// Returns [`VmmError::VmaNotFound`] if no VMA starting at `virt` exists
    pub unsafe fn remap_region(
        address_space: &mut AddressSpace,
        virt: VirtAddr,
        new_vma_flags: VmaFlags
    ) -> VmmResult
    where
        Self: VmmPaging
    {
        #[cfg(debug_assertions)]
        if !virt.is_aligned(FRAME_SIZE) {
            kernel_panic(
                PanicCode::MisalignedAddress,
                "Vmm::remap_region recieved an improperly aligned virtual address or size"
            );
        }

        let mut vma = address_space.remove_vma(virt).ok_or(VmmError::VmaNotFound)?;
        let size = vma.end_addr.as_u64() - vma.start_addr.as_u64();

        vma.flags = new_vma_flags;
        address_space.vmas.insert(vma);

        let new_page_flags = Self::vma_flags_to_page_flags(new_vma_flags);

        let mut offset = 0;
        while offset < size {
            if let Some((_, page_size)) = Self::translate_with_size(address_space.page_ptr, virt + offset) {
                unsafe { Self::remap_page(address_space.page_ptr, virt + offset, new_page_flags); }
                offset += page_size;
            } else {
                offset += FRAME_SIZE;
            }
        }

        Ok(())
    }
}
