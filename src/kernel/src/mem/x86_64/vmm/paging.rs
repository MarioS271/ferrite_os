// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging: owns a page table, maps, unmaps, remaps and translates pages.
//!
//! Authors: MarioS271

use core::fmt::{Display, Formatter};
use x86_64::structures::paging::{PageTable, PageTableFlags, PageTableIndex, PhysFrame};
use x86_64::instructions::tlb;
use x86_64::structures::paging::page_table::PageTableEntry;
use super::vmm::{Vmm, alloc_zeroed_frame};
use crate::mem::x86_64::pmm::{Pmm, FRAME_SIZE};
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::addr::{PhysAddr, VirtAddr};
use crate::types::panic_codes::PanicCode;

const HUGE_PAGE_SIZE_2MIB: u64 = FRAME_SIZE * 512;
const HUGE_PAGE_SIZE_1GIB: u64 = HUGE_PAGE_SIZE_2MIB * 512;

#[repr(u64)]
#[derive(Copy, Clone, PartialEq)]
pub enum PageType {
    Normal = FRAME_SIZE,
    HugePage2MiB = HUGE_PAGE_SIZE_2MIB,
    HugePage1GiB = HUGE_PAGE_SIZE_1GIB
}
impl Display for PageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", match self {
            PageType::Normal => "Normal(4KiB)",
            PageType::HugePage2MiB => "HugePage(2MiB)",
            PageType::HugePage1GiB => "HugePage(1GiB)",
        })
    }
}

impl Vmm {
    /// Map one 4 KiB virtual page to a physical frame. `PRESENT` is forced on regardless of `flags`.
    ///
    /// # Safety
    /// The caller must ensure `virt` is a valid kernel virtual address and `phys` is a
    /// valid, PMM-allocated frame.
    ///
    /// # Panics
    /// Panics if the PMM runs out of frames when allocating an intermediate page table.
    pub unsafe fn map_page(&self, pmm: &mut Pmm, virt: VirtAddr, phys: PhysAddr, page_type: PageType, flags: PageTableFlags) {
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

        let mut current_pagetable: *mut PageTable = self.plm4_ptr;
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
            misaligned_address_panic(PageType::Normal);
        }

        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut current_pagetable: *mut PageTable = self.plm4_ptr;

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

    /// Change page table flags of an already mapped page
    ///
    /// # Safety
    /// The caller must ensure `virt` was previously mapped and that changing the page's flags
    /// will not violate anything (example: making a page non-writable while a mutable reference
    /// is held to it)
    ///
    /// # Panics
    /// Panics if any level of the walk is not present (page was never mapped).
    pub unsafe fn remap_page(&self, virt: VirtAddr, new_flags: PageTableFlags) {
        if virt.as_u64() % FRAME_SIZE != 0 {
            misaligned_address_panic(PageType::Normal);
        }

        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut current_pagetable: *mut PageTable = self.plm4_ptr;

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

    /// Walk the page table and return the phys address mapped at `virt` or `None` if any
    /// level is not present
    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let mut current = self.plm4_ptr;

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
            return Some(PhysAddr::new(entry.addr().as_u64() + calc_offset(HUGE_PAGE_SIZE_1GIB - 1)));
        }
        current = advance_current_pagetable(entry);

        // PD
        let entry = &unsafe { core::ptr::read(current) }[virt.p2_index()];
        if !is_present(entry) { return None; }
        if is_huge_page(entry) {
            return Some(PhysAddr::new(entry.addr().as_u64() + calc_offset(HUGE_PAGE_SIZE_2MIB - 1)));
        }
        current = advance_current_pagetable(entry);

        // PT
        let entry = &unsafe { core::ptr::read(current) }[virt.p1_index()];
        if !is_present(entry) { return None; }
        Some(PhysAddr::new(entry.addr().as_u64() + calc_offset(FRAME_SIZE - 1)))
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

/// Panic when `unmap_page` hits a page-table level that is not present.
pub fn invalid_unmap_panic() -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        "Attempting to unmap a page without a PRESENT flag",
    );
}

/// Panic when `remap_page` hits a page-table level that is not present.
pub fn invalid_remap_panic() -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        "Attempting to remap a page without a PRESENT flag",
    );
}
