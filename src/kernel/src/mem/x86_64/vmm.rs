//! mem/vmm.rs
//! Virtual Memory Manager
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::kprint;
use crate::mem::pmm;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use spin::Once;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, PageTableFlags, PageTableIndex, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

static VMM_DATA: Once<VmmData> = Once::new();

pub struct VmmData {
    plm4_ptr: *mut PageTable,
    hhdm_offset: u64,
}

// Safe because all access is controlled via the Once
unsafe impl Send for VmmData {}
unsafe impl Sync for VmmData {}

fn alloc_zeroed_frame() -> PhysFrame {
    let frame = pmm::alloc().unwrap_or_else(
        || kernel_panic(
            PanicCode::OutOfMemory,
            "VMM could not allocate a frame, out of memory",
            true
        )
    );

    // Safe because the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes((frame.as_u64() + get().hhdm_offset) as *mut u64, 0x00, 1);
    }

    PhysFrame::from_start_address(PhysAddr::new(frame.as_u64())).unwrap()
}

pub fn init(hhdm_offset: u64) {
    let limine_plm4_ptr = (Cr3::read().0.start_address().as_u64() + hhdm_offset) as *const PageTable;

    let plm4_ptr = (pmm::alloc().unwrap_or_else(
        || kernel_panic(
            PanicCode::OutOfMemory,
            "VMM could not allocate PLM4 page, out of memory",
            true
        )
    ).as_u64() + hhdm_offset) as *mut PageTable;

    // Safe because the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes(plm4_ptr, 0x00, 1);
    }

    for entry in 256..512 {
        // Both pointers are valid and only touched here at time of execution
        unsafe {
            (&mut (*plm4_ptr))[entry] = (&(*limine_plm4_ptr))[entry].clone();
        }
    }

    let phys_addr_u64 = plm4_ptr as u64 - hhdm_offset;
    let phys_frame = PhysFrame::containing_address(PhysAddr::new(phys_addr_u64));
    let current_cr3_flags = Cr3::read().1;

    // Using the same plm4 as provided by limine before, just copied so that the kernel
    // is able to own it (limine's plm4 was safe and functional)
    unsafe {
        Cr3::write(phys_frame, current_cr3_flags);
    }

    kprint!("[VMM] allocated frame for plm4 at phys addr {phys_addr_u64:#x}\n");

    VMM_DATA.call_once(|| VmmData{
        plm4_ptr,
        hhdm_offset,
    });
}

pub fn get() -> &'static VmmData {
    VMM_DATA.get().unwrap()
}

// Unsafe func because memory will be modified by caller-given data
pub unsafe fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageTableFlags) {
    let mut current_pagetable: *mut PageTable = get().plm4_ptr;
    let intermediate_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | (flags & PageTableFlags::USER_ACCESSIBLE);

    for level in (2..5).rev() {
        let index: PageTableIndex;

        match level {
            4 => { index = virt.p4_index() }
            3 => { index = virt.p3_index() }
            2 => { index = virt.p2_index() }
            _ => unreachable!()
        }

        let entry = &mut (&mut (*current_pagetable))[index];

        if !entry.flags().contains(PageTableFlags::PRESENT) {
            let frame = alloc_zeroed_frame();
            entry.set_frame(frame, intermediate_flags);
        }

        current_pagetable = (entry.frame().unwrap().start_address().as_u64() + get().hhdm_offset) as *mut PageTable;
    }

    let entry = &mut (&mut (*current_pagetable))[virt.p1_index()];
    entry.set_frame(PhysFrame::containing_address(phys), flags | PageTableFlags::PRESENT);
}