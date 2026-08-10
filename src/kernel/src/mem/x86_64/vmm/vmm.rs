// SPDX-License-Identifier: GPL-3.0-only
//! Virtual Memory Manager (VMM) for x86_64: owns the kernel page table and maps
//! and unmaps pages at 4 KiB granularity.
//!
//! Authors: MarioS271

use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, PageTableFlags, PageTableIndex, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::instructions::tlb;
use x86_64::structures::paging::page_table::PageTableEntry;
use super::vmm_helpers::*;
use crate::{kdebug, kinfo};
use crate::mem::x86_64::pmm::{Pmm, FRAME_SIZE};
use crate::types::irq_mutex::IrqMutex;

/// Kernel page-table state used by all VMM operations.
pub struct Vmm {
    pub plm4_ptr: *mut PageTable,
    pub hhdm_offset: u64,
}

// Safe because Vmm is written once during init (single-threaded) and then
// only read after that.
unsafe impl Send for Vmm {}
unsafe impl Sync for Vmm {}

impl Vmm {
    /// Initialize the VMM and install a kernel-owned PML4 into CR3.
    ///
    /// # Panics
    /// Panics if the PMM cannot allocate the PML4 frame (out of memory).
    pub fn init(pmm_mutex: &IrqMutex<Pmm>, hhdm_offset: u64) -> Self {
        let mut pmm = pmm_mutex.lock();

        let limine_plm4_ptr = (Cr3::read().0.start_address().as_u64() + hhdm_offset) as *const PageTable;

        let plm4_ptr = (
            pmm.alloc_frame().unwrap_or_else(|| out_of_memory_panic()).as_u64() + hhdm_offset
        ) as *mut PageTable;

        // Safe because the PMM gives us a valid piece of memory
        unsafe {
            core::ptr::write_bytes(plm4_ptr, 0x00, 1);
        }

        unsafe {
            core::ptr::copy_nonoverlapping(
                (limine_plm4_ptr as *const PageTableEntry).add(256),
                (plm4_ptr as *mut PageTableEntry).add(256),
                256
            );
        }

        let phys_addr_u64 = plm4_ptr as u64 - hhdm_offset;
        let phys_frame = PhysFrame::containing_address(PhysAddr::new(phys_addr_u64));
        let current_cr3_flags = Cr3::read().1;

        // Using the same plm4 as provided by limine before, just copied so that the kernel
        // is able to own it (limine's plm4 was safe and functional)
        unsafe {
            Cr3::write(phys_frame, current_cr3_flags);
        }

        kdebug!("[VMM] allocated frame for plm4 at phys addr {phys_addr_u64:#x}");
        kinfo!("Initialized VMM");

        Vmm { plm4_ptr, hhdm_offset }
    }

    /// Map one 4 KiB virtual page to a physical frame. `PRESENT` is forced on regardless of `flags`.
    ///
    /// # Safety
    /// The caller must ensure `virt` is a valid kernel virtual address and `phys` is a
    /// valid, PMM-allocated frame.
    ///
    /// # Panics
    /// Panics if the PMM runs out of frames when allocating an intermediate page table.
    pub unsafe fn map_page(&self, pmm: &mut Pmm, virt: VirtAddr, phys: PhysAddr, flags: PageTableFlags) {
        if phys.as_u64() % FRAME_SIZE != 0 || virt.as_u64() % FRAME_SIZE != 0 {
            misaligned_address_panic_4kib();
        }

        let mut current_pagetable: *mut PageTable = self.plm4_ptr;
        let intermediate_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | (flags & PageTableFlags::USER_ACCESSIBLE);

        for level in (2..5).rev() {
            let index: PageTableIndex = match level {
                4 => virt.p4_index(),
                3 => virt.p3_index(),
                2 => virt.p2_index(),
                _ => unreachable!()
            };

            let entry = &mut current_pagetable.as_mut().unwrap()[index];

            if !entry.flags().contains(PageTableFlags::PRESENT) {
                let frame = alloc_zeroed_frame(pmm, self.hhdm_offset);
                entry.set_frame(frame, intermediate_flags);
            }

            current_pagetable = (entry.frame().unwrap().start_address().as_u64() + self.hhdm_offset) as *mut PageTable;
        }

        let entry = &mut current_pagetable.as_mut().unwrap()[virt.p1_index()];
        entry.set_frame(PhysFrame::containing_address(phys), flags | PageTableFlags::PRESENT);
    }

    /// Unmap one 4 KiB virtual page and flush its TLB entry.
    ///
    /// # Safety
    /// The caller must ensure `virt` was previously mapped and that no live code or
    /// data references the page after this call returns.
    ///
    /// # Panics
    /// Panics if any level of the walk is not present (page was never mapped).
    pub unsafe fn unmap_page(&self, virt: VirtAddr) {
        if virt.as_u64() % FRAME_SIZE != 0 {
            misaligned_address_panic_4kib();
        }

        let mut current_pagetable: *mut PageTable = self.plm4_ptr;

        for level in (2..5).rev() {
            let index: PageTableIndex = match level {
                4 => virt.p4_index(),
                3 => virt.p3_index(),
                2 => virt.p2_index(),
                _ => unreachable!()
            };

            let entry = &mut current_pagetable.as_mut().unwrap()[index];

            if !entry.flags().contains(PageTableFlags::PRESENT) {
                invalid_unmap_panic();
            }

            current_pagetable = (entry.frame().unwrap().start_address().as_u64() + self.hhdm_offset) as *mut PageTable;
        }

        let entry = &mut current_pagetable.as_mut().unwrap()[virt.p1_index()];

        if !entry.flags().contains(PageTableFlags::PRESENT) {
            invalid_unmap_panic();
        }

        entry.set_unused();
        tlb::flush(virt);
    }
}
