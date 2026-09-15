// SPDX-License-Identifier: GPL-3.0-only
//! Stores the page pointer and VMAs for a process or the kernel
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::mm::pmm::FRAME_SIZE;
use crate::mm::vmm::boot_mapping::BootMappings;
use crate::mm::vmm::traits::VmmPaging;
use crate::mm::vmm::vma::Vma;
use crate::mm::vmm::{Vmm, VmmError, VmmResult};
use crate::state::kstate::KSTATE;
use alloc::collections::BTreeSet;

/// Type to represent the memory of a process or the kernel by holding a pointer to its page tables and VMAs
pub struct AddressSpace {
    page_ptr: VirtAddr,
    vmas: BTreeSet<Vma>,
}

impl AddressSpace {
    /// Creates a new `AddressSpace` instance which contains the given page table pointer and no VMAs
    pub fn empty(page_ptr: VirtAddr) -> Self {
        Self {
            page_ptr,
            vmas: BTreeSet::new()
        }
    }

    /// Create a new address space with a zeroed root page, and copy the kernel mappings into the
    /// higher half
    ///
    /// # Safety
    /// The caller must guarantee that this method is only called post-stage-2 when memory management
    /// has already been initialized
    pub unsafe fn new_user_addr_space() -> Self {
        let frame = unsafe { KSTATE.mm.pmm() }.lock().alloc_frame().unwrap_or_else(
            || kernel_panic(
                PanicCode::OutOfMemory,
                "Could not allocate a root page for the a address space, out of memory"
            )
        );

        let page = VirtAddr::from_phys(frame);
        // Safety: page is a valid frame; also, this assumes that one root page is exactly 4 KiB
        // (or one FRAME_SIZE) large, if it is not, this code will silently zero the wrong amount
        // of memory!
        unsafe {
            core::ptr::write_bytes(page.as_mut_ptr::<u8>(), 0u8, FRAME_SIZE as usize);
            Vmm::clone_kernel_mappings(page);
        }

        AddressSpace::empty(page)
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
    pub fn setup_kernel_vmas(&mut self, boot_mappings: &BootMappings) -> VmmResult {
        for mapping in boot_mappings {
            self.insert_vma(
                Vma {
                    start_addr: mapping.virt,
                    end_addr: mapping.virt + mapping.size,
                    flags: mapping.flags,
                }
            )?;
        }

        Ok(())
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

    /// Search for a VMA-free range of `size` bytes in `[from, to)`
    /// Returns the lowest address where such a range fits, or [`None`] if such a
    /// range couldn't be found between VMAs
    pub fn find_free_range(&self, from: VirtAddr, to: VirtAddr, size: u64) -> Option<VirtAddr> {
        let mut candidate = from;

        for vma in self.vmas.range(from..) {
            if vma.start_addr >= to {
                break;
            }

            if vma.start_addr.as_u64() - candidate.as_u64() >= size {
                return Some(candidate);
            }

            if vma.end_addr > candidate {
                candidate = vma.end_addr;
            }
        }

        if to.as_u64() - candidate.as_u64() >= size {
            Some(candidate)
        } else {
            None
        }
    }
}
