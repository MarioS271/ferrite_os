// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging: owns a page table, maps, unmaps, remaps and translates pages.
//!
//! Authors: MarioS271

use super::helpers::alloc_zeroed_frame;
use super::page_type::{PageType, HUGE_PAGE_SIZE_1GIB, HUGE_PAGE_SIZE_2MIB};
use crate::kinfo;
use crate::mem::pmm::{Pmm, FRAME_SIZE};
use crate::mem::vmm::helpers::{invalid_remap_panic, invalid_unmap_panic, out_of_memory_panic};
use crate::mem::vmm::traits::VmmPaging;
use crate::mem::vmm::vma::VmaFlags;
use crate::mem::vmm::Vmm;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::addr::{PhysAddr, VirtAddr};
use crate::types::panic_codes::PanicCode;
use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{PageTable, PageTableFlags, PageTableIndex, PhysFrame};

impl VmmPaging for Vmm {
    type PageType = PageType;
    type PageTableFlags = PageTableFlags;

    fn setup_kernel_page() -> VirtAddr {
        // TODO: do own paging

        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut pmm = KSTATE.mm.pmm().lock();

        let limine_page_ptr = (Cr3::read().0.start_address().as_u64() + hhdm_offset) as *const PageTable;

        let kernel_page_ptr = (
            pmm.alloc_frame().unwrap_or_else(|| out_of_memory_panic()).as_u64() + hhdm_offset
        ) as *mut PageTable;

        // Safe because the PMM gives us a valid piece of memory
        unsafe {
            core::ptr::write_bytes(kernel_page_ptr, 0x00, 1);
        }

        unsafe {
            core::ptr::copy_nonoverlapping(
                (limine_page_ptr as *const PageTableEntry).add(256),
                (kernel_page_ptr as *mut PageTableEntry).add(256),
                256
            );
        }

        let phys_addr_u64 = kernel_page_ptr as u64 - hhdm_offset;
        let phys_frame = PhysFrame::containing_address(x86_64::PhysAddr::new(phys_addr_u64));
        let current_cr3_flags = Cr3::read().1;

        // Using the same PML4 as provided by limine before, just copied so that the kernel
        // is able to own it (limine's plm4 was safe and functional)
        unsafe {
            Cr3::write(phys_frame, current_cr3_flags);
        }

        kinfo!("Initialized kernel PML4 (Phys Addr: {phys_addr_u64:#x})");

        VirtAddr::new(kernel_page_ptr as u64)
    }

    unsafe fn map_page(
        pmm: &mut Pmm,
        page_ptr: VirtAddr,
        virt: VirtAddr,
        phys: PhysAddr,
        page_type: PageType,
        flags: PageTableFlags
    ) {
        let align = &(page_type as u64);
        if phys.as_u64() % align != 0 || virt.as_u64() % align != 0 {
            misaligned_address_panic(page_type);
        }

        let hhdm_offset = &KSTATE.mm.hhdm_offset();

        let lowest_iter_page = match page_type {
            PageType::Normal => 2,
            PageType::HugePage2MiB => 3,
            PageType::HugePage1GiB => 4
        };

        let mut current_pagetable: *mut PageTable = page_ptr.as_mut_ptr::<PageTable>();
        let intermediate_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | (flags & PageTableFlags::USER_ACCESSIBLE);

        for level in (lowest_iter_page..=4).rev() {
            let index: PageTableIndex = match level {
                4 => virt.p4_index(),
                3 => virt.p3_index(),
                2 => virt.p2_index(),
                _ => unreachable!()
            };

            let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[index] };

            if !entry.flags().contains(PageTableFlags::PRESENT) {
                let frame = alloc_zeroed_frame(pmm);
                entry.set_frame(frame, intermediate_flags);
            }

            current_pagetable = (entry.frame().unwrap().start_address().as_u64() + hhdm_offset) as *mut PageTable;
        }

        // Safe: current_pagetable is a valid
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[match page_type {
            PageType::Normal => virt.p1_index(),
            PageType::HugePage2MiB => virt.p2_index(),
            PageType::HugePage1GiB => virt.p3_index(),
        }] };

        if page_type == PageType::Normal {
            entry.set_frame(PhysFrame::containing_address(phys.as_x86_64()), flags | PageTableFlags::PRESENT);
        } else {
            entry.set_addr(phys.as_x86_64(), flags | PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE);
        }

        #[cfg(feature = "vmm-debug-logging")]
        crate::kdebug!("[VMM] map new page at virt {virt:#x}, phys {phys:#x}, type {page_type}, flags {flags:?}");
    }

    unsafe fn unmap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) {
        if virt.as_u64() % FRAME_SIZE != 0 {
            misaligned_address_panic(PageType::Normal);
        }

        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut current_pagetable: *mut PageTable = page_ptr.as_mut_ptr::<PageTable>();

        let is_huge_page = |entry: &PageTableEntry| -> bool {
            entry.flags().contains(PageTableFlags::HUGE_PAGE)
        };
        let assert_is_present = |entry: &mut PageTableEntry| {
            if !entry.flags().contains(PageTableFlags::PRESENT) { invalid_unmap_panic(); }
        };
        let advance_current_pagetable = |entry: &mut PageTableEntry| -> *mut PageTable {
            (entry.frame().unwrap().start_address().as_u64() + hhdm_offset) as *mut PageTable
        };
        let clear_and_flush = |entry: &mut PageTableEntry| {
            entry.set_unused();
            tlb::flush(virt.as_x86_64());
        };
        let debug_log = |virt: &VirtAddr| {
            #[cfg(feature = "vmm-debug-logging")]
            crate::kdebug!("[VMM] unmap page at virt {virt:#x}");
        };

        // UNSAFE JUSTIFICATION 4x for below
        // The pointer which is being read (current) is a known and correct addr

        // PML4
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p4_index()] };
        assert_is_present(entry);
        current_pagetable = advance_current_pagetable(entry);

        // PDPT
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p3_index()] };
        assert_is_present(entry);
        if is_huge_page(entry) {
            clear_and_flush(entry);
            debug_log(&virt);
            return;
        }
        current_pagetable = advance_current_pagetable(entry);

        // PD
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p2_index()] };
        assert_is_present(entry);
        if is_huge_page(entry) {
            clear_and_flush(entry);
            debug_log(&virt);
            return;
        }
        current_pagetable = advance_current_pagetable(entry);

        // PT
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p1_index()] };
        assert_is_present(entry);
        clear_and_flush(entry);
        debug_log(&virt);
    }

    unsafe fn remap_page(
        page_ptr: VirtAddr,
        virt: VirtAddr,
        new_flags: PageTableFlags
    ) {
        if virt.as_u64() % FRAME_SIZE != 0 {
            misaligned_address_panic(PageType::Normal);
        }

        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut current_pagetable: *mut PageTable = page_ptr.as_mut_ptr::<PageTable>();

        let is_huge_page = |entry: &PageTableEntry| -> bool {
            entry.flags().contains(PageTableFlags::HUGE_PAGE)
        };
        let assert_is_present = |entry: &mut PageTableEntry| {
            if !entry.flags().contains(PageTableFlags::PRESENT) { invalid_remap_panic(); }
        };
        let advance_current_pagetable = |entry: &mut PageTableEntry| -> *mut PageTable {
            (entry.frame().unwrap().start_address().as_u64() + hhdm_offset) as *mut PageTable
        };
        let flush_tlb = || {
            tlb::flush(virt.as_x86_64());
        };
        let debug_log = |virt: &VirtAddr, flags: &PageTableFlags| {
            #[cfg(feature = "vmm-debug-logging")]
            crate::kdebug!("[VMM] remap page at virt {virt:#x} to flags {flags:?}");
        };

        // UNSAFE JUSTIFICATION 4x for below
        // The pointer which is being read (current) is a known and correct addr

        // PML4
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p4_index()] };
        assert_is_present(entry);
        current_pagetable = advance_current_pagetable(entry);

        // PDPT
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p3_index()] };
        assert_is_present(entry);
        if is_huge_page(entry) {
            entry.set_addr(entry.addr(), new_flags | PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE);
            flush_tlb();
            debug_log(&virt, &new_flags);
            return;
        }
        current_pagetable = advance_current_pagetable(entry);

        // PD
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p2_index()] };
        assert_is_present(entry);
        if is_huge_page(entry) {
            entry.set_addr(entry.addr(), new_flags | PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE);
            flush_tlb();
            debug_log(&virt, &new_flags);
            return;
        }
        current_pagetable = advance_current_pagetable(entry);

        // PT
        let entry = unsafe { &mut current_pagetable.as_mut().unwrap()[virt.p1_index()] };
        assert_is_present(entry);
        entry.set_frame(
            PhysFrame::containing_address(entry.frame().unwrap().start_address()),
            new_flags | PageTableFlags::PRESENT
        );
        flush_tlb();
        debug_log(&virt, &new_flags);
    }

    fn translate(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> Option<PhysAddr> {
        let res = Self::translate_with_size(page_ptr, virt)?;
        Some(res.0)
    }

    fn translate_with_size(
        page_ptr: VirtAddr,
        virt: VirtAddr
    ) -> Option<(PhysAddr, u64)> {
        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut current = page_ptr.as_mut_ptr::<PageTable>();

        let is_huge_page = |entry: &PageTableEntry| -> bool {
            entry.flags().contains(PageTableFlags::HUGE_PAGE)
        };
        let is_present = |entry: &PageTableEntry| -> bool {
            entry.flags().contains(PageTableFlags::PRESENT)
        };
        let advance_current_pagetable = |entry: &PageTableEntry| -> *mut PageTable {
            (entry.frame().unwrap().start_address().as_u64() + hhdm_offset) as *mut PageTable
        };
        let calc_offset = |mask: u64| -> u64 {
            virt.as_u64() & mask
        };

        // UNSAFE JUSTIFICATION 4x for below
        // The pointer which is being read (current) is a known and correct addr

        // PML4
        let entry = &unsafe { core::ptr::read(current) }[virt.p4_index()];
        if !is_present(entry) { return None; }
        current = advance_current_pagetable(entry);

        // PDPT
        let entry = &unsafe { core::ptr::read(current) }[virt.p3_index()];
        if !is_present(entry) { return None; }
        if is_huge_page(entry) {
            return Some(
                (PhysAddr::new(entry.addr().as_u64() + calc_offset(HUGE_PAGE_SIZE_1GIB - 1)), HUGE_PAGE_SIZE_1GIB)
            );
        }
        current = advance_current_pagetable(entry);

        // PD
        let entry = &unsafe { core::ptr::read(current) }[virt.p2_index()];
        if !is_present(entry) { return None; }
        if is_huge_page(entry) {
            return Some(
                (PhysAddr::new(entry.addr().as_u64() + calc_offset(HUGE_PAGE_SIZE_2MIB - 1)), HUGE_PAGE_SIZE_2MIB)
            );
        }
        current = advance_current_pagetable(entry);

        // PT
        let entry = &unsafe { core::ptr::read(current) }[virt.p1_index()];
        if !is_present(entry) { return None; }
        Some(
            (PhysAddr::new(entry.addr().as_u64() + calc_offset(FRAME_SIZE - 1)), FRAME_SIZE)
        )
    }

    fn vma_flags_to_page_flags(
        vma_flags: VmaFlags
    ) -> Self::PageTableFlags {
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
}

/// Panic when a given address is not aligned properly
pub fn misaligned_address_panic(align: PageType) -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        match align {
            PageType::Normal => "Physical and/or virtual address is not aligned to 4 KiB",
            PageType::HugePage2MiB => "Physical and/or virtual address is not aligned to 2 MiB",
            PageType::HugePage1GiB => "Physical and/or virtual address is not aligned to 1 GiB"
        }
    );
}
