// SPDX-License-Identifier: GPL-3.0-only
//! Stores the PML4 pointer and VMAs for a process or the kernel
//!
//! Authors: MarioS271

use alloc::collections::BTreeSet;
use crate::mem::vma::Vma;
use crate::types::addr::VirtAddr;

/// Type to represent the memory of a process or the kernel by holding a pointer to its page tables and VMAs
pub struct AddressSpace {
    pub root_table_ptr: VirtAddr,
    pub vmas: BTreeSet<Vma>,
}

impl AddressSpace {
    /// Creates a new `AddressSpace` instance which contains the given page table pointer and no VMAs
    pub fn new(root_table_ptr: VirtAddr) -> Self {
        Self {
            root_table_ptr,
            vmas: BTreeSet::new()
        }
    }

    /// Getter for [`Vma::root_table_ptr`]
    pub fn get_root_table_ptr(&self) -> VirtAddr {
        self.root_table_ptr
    }

    /// Add a VMA to the `AddressSpace`
    pub fn insert_vma(&mut self, vma: Vma) {
        self.vmas.insert(vma);
    }

    /// Remove a VMA from the `AddressSpace`
    /// Returns whether the element to remove existed and could be removed or not
    pub fn remove_vma(&mut self, vma_start_addr: VirtAddr) -> bool {
        self.vmas.remove(&vma_start_addr)
    }

    /// Search for a VMA in the `AddressSpace` which applies VMA flags to the given `VirtAddr`
    /// Returns a reference to the VMA wrapped in an option incase it is not found
    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        self.vmas.range(..=addr).next_back()
    }
}
