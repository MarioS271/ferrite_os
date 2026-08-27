// SPDX-License-Identifier: GPL-3.0-only
//! Stores the page pointer and VMAs for a process or the kernel
//!
//! Authors: MarioS271

use crate::mem::vmm::vma::Vma;
use crate::mem::vmm::VmmError;
use crate::types::addr::VirtAddr;
use alloc::collections::BTreeSet;

/// Type to represent the memory of a process or the kernel by holding a pointer to its page tables and VMAs
pub struct AddressSpace {
    pub page_ptr: VirtAddr,
    pub vmas: BTreeSet<Vma>,
}

impl AddressSpace {
    /// Creates a new `AddressSpace` instance which contains the given page table pointer and no VMAs
    pub fn new(page_ptr: VirtAddr) -> Self {
        Self {
            page_ptr,
            vmas: BTreeSet::new()
        }
    }

    /// Getter for [`AddressSpace::page_ptr`]
    pub fn get_page_ptr(&self) -> VirtAddr {
        self.page_ptr
    }

    /// Add a VMA to the `AddressSpace`
    pub fn insert_vma(&mut self, vma: Vma) -> Result<(), VmmError> {
        for vma_other in &self.vmas {
            if vma_other.overlaps(&vma) {
                return Err(VmmError::VmaOverlap);
            }
        }
        self.vmas.insert(vma);
        Ok(())
    }

    /// Remove a VMA from the `AddressSpace`
    /// Returns whether the element to remove existed and could be removed or not
    pub fn remove_vma(&mut self, vma_start_addr: VirtAddr) -> bool {
        self.vmas.remove(&vma_start_addr)
    }

    /// Search for a VMA in the `AddressSpace` which applies VMA flags to the given `VirtAddr`
    /// Returns a reference to the VMA wrapped in an option incase it is not found
    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        self.vmas.range(..=addr).next_back().filter(|vma| vma.contains(addr))
    }
}
