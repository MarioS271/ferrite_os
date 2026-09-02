// SPDX-License-Identifier: GPL-3.0-only
//! Stores the page pointer and VMAs for a process or the kernel
//!
//! Authors: MarioS271

use crate::mem::vmm::vma::{Vma, VmaFlags};
use crate::mem::vmm::{VmmError, VmmResult};
use crate::state::kstate::KSTATE;
use crate::types::addr::{PhysAddr, VirtAddr};
use alloc::collections::BTreeSet;

/// Type to represent the memory of a process or the kernel by holding a pointer to its page tables and VMAs
pub struct AddressSpace {
    page_ptr: VirtAddr,
    vmas: BTreeSet<Vma>,
}

impl AddressSpace {
    /// Creates a new `AddressSpace` instance which contains the given page table pointer and no VMAs
    pub fn new(page_ptr: VirtAddr) -> Self {
        Self {
            page_ptr,
            vmas: BTreeSet::new()
        }
    }

    /// Getter for `AddressSpace::page_ptr`
    pub fn page_ptr(&self) -> VirtAddr {
        self.page_ptr
    }

    /// Getter for `AddressSpace::vmas`
    pub fn vmas(&self) -> &BTreeSet<Vma> {
        &self.vmas
    }

    /// Initializes Kernel VMAs on self
    /// Do not call this ever, unless you are initializing the kernel's address space
    pub fn setup_kernel_vmas(&mut self) {
        // HHDM
        let _ = self.insert_vma(
            Vma {
                start_addr: VirtAddr::new(KSTATE.mm.hhdm_offset()),
                end_addr: VirtAddr::from_phys(PhysAddr::new(
                    // Safety: the PMM gets initialized before any address space does
                    unsafe { KSTATE.mm.pmm().lock().get_total_mem() }
                )),
                flags: VmaFlags::READ | VmaFlags::WRITE
            }
        );
        // Kernel .text
        let _ = self.insert_vma(
            Vma {
                start_addr: VirtAddr::new(&raw const crate::__kernel_start as u64),
                end_addr: VirtAddr::new(&raw const crate::__kernel_text_end as u64),
                flags: VmaFlags::READ | VmaFlags::EXEC
            }
        );
        // Kernel .rodata
        let _ = self.insert_vma(
            Vma {
                start_addr: VirtAddr::new(&raw const crate::__kernel_text_end as u64),
                end_addr: VirtAddr::new(&raw const crate::__kernel_rodata_end as u64),
                flags: VmaFlags::READ
            }
        );
        // Kernel .data and .bss
        let _ = self.insert_vma(
            Vma {
                start_addr: VirtAddr::new(&raw const crate::__kernel_rodata_end as u64),
                end_addr: VirtAddr::new(&raw const crate::__kernel_end as u64),
                flags: VmaFlags::READ | VmaFlags::WRITE
            }
        );
    }

    /// Add a VMA to the `AddressSpace`
    pub fn insert_vma(&mut self, vma: Vma) -> VmmResult {
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
    pub fn remove_vma(&mut self, vma_start_addr: VirtAddr) -> Option<Vma> {
        self.vmas.take(&vma_start_addr)
    }

    /// Search for a VMA in the `AddressSpace` which applies VMA flags to the given `VirtAddr`
    /// Returns a reference to the VMA wrapped in an option incase it is not found
    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        self.vmas.range(..=addr).next_back().filter(|vma| vma.contains(addr))
    }
}
