// SPDX-License-Identifier: GPL-3.0-only
//! Stores the page pointer and VMAs for a process or the kernel
//!
//! Authors: MarioS271

use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::boot_info::KernelSectionInfo;
use crate::mm::vmm::traits::VmmPaging;
use crate::mm::vmm::vma::{Vma, VmaFlags};
use crate::mm::vmm::{Vmm, VmmError, VmmResult};
use crate::state::kstate::KSTATE;
use alloc::collections::BTreeSet;
use crate::mm::pmm::FRAME_SIZE;

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
    pub fn new_user_addr_space() -> Self {
        // Safety: the PMM was already initialized in stage 2
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
    pub fn setup_kernel_vmas(&mut self, section_info: &KernelSectionInfo) {
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
                start_addr: VirtAddr::new(section_info.kernel_start),
                end_addr: VirtAddr::new(section_info.kernel_text_end),
                flags: VmaFlags::READ | VmaFlags::EXEC
            }
        );
        // Kernel .rodata
        let _ = self.insert_vma(
            Vma {
                start_addr: VirtAddr::new(section_info.kernel_text_end),
                end_addr: VirtAddr::new(section_info.kernel_rodata_end),
                flags: VmaFlags::READ
            }
        );
        // Kernel .data and .bss
        let _ = self.insert_vma(
            Vma {
                start_addr: VirtAddr::new(section_info.kernel_rodata_end),
                end_addr: VirtAddr::new(section_info.kernel_end),
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
