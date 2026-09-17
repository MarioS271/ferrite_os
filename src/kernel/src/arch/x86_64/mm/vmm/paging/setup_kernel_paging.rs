// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `setup_kernel_paging`
//!
//! Authors: MarioS271

use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::boot_info::KernelSectionInfo;
use crate::mm::layout;
use crate::mm::pmm::{Pmm, FRAME_SIZE};
use crate::mm::vmm::boot_mapping::{build_boot_mappings, BootMappings};
use crate::mm::vmm::traits::VmmPaging;
use crate::mm::vmm::Vmm;
use crate::state::kstate::KSTATE;
use crate::kinfo;
use x86_64::registers::control::{Cr3, Efer, EferFlags};
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame};

// TODO: add KASLR

/// Build the kernels page tables and switch to them
pub fn setup_kernel_paging(sections: &KernelSectionInfo) -> (VirtAddr, BootMappings) {
    // Safety: NXE is supported on all long mode capable CPUs
    unsafe {
        Efer::update(|efer| efer.insert(EferFlags::NO_EXECUTE_ENABLE));
    }

    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

    let pml4_frame = pmm.alloc_frame_zeroed().unwrap_or_else(
        || kernel_panic(
            PanicCode::OutOfMemory,
            "Could not allocate a PML4 for the kernel, out of memory"
        )
    );

    let page_ptr = VirtAddr::from_phys(pml4_frame);
    let total_mem = pmm.get_total_mem();

    let boot_mappings = build_boot_mappings(sections, total_mem);

    for mapping in &boot_mappings {
        // Safety: page_ptr is a freshly allocated frame and the phys ranges come from the
        // memorymap and the loaded kernel image where nothing is mapped yet
        unsafe {
            Vmm::map_range(
                &mut pmm,
                page_ptr,
                mapping.virt,
                mapping.phys,
                mapping.size,
                Vmm::vma_flags_to_page_flags(mapping.flags),
                mapping.can_overmap
            ).unwrap_or_else(
                |_| kernel_panic(
                    PanicCode::OutOfMemory,
                    "Could not build the kernel page tables, out of memory"
                )
            );
        }
    }

    create_zeroed_pdpts(&mut pmm, page_ptr);
    verify_against_bootloader(page_ptr, sections);

    // Safety: everything the kernel is currently executing and/or working with has been mapped and
    // verified in the new pages and will resolve the same as before
    unsafe {
        Cr3::write(
            PhysFrame::containing_address(pml4_frame.as_x86_64()),
            Cr3::read().1
        );
    }

    kinfo!("Initialized kernel paging (PML4 is at phys {:#x})", pml4_frame.as_u64());

    (page_ptr, boot_mappings)
}

/// Prepopulate the higher half with empty PDPTs so that if the kernel PML4 higher half ever gets
/// cloned (like when creating processes), no stale higher halfs exist that aren't synced with the
/// kernel's (which could cause page faults when switching into the kernel with user pages)
///
/// TODO: only create a zeroed pdpt for areas where we actually need them
fn create_zeroed_pdpts(pmm: &mut Pmm, page_ptr: VirtAddr) {
    // Safety: page_ptr points to a newly allocated and zeroed frame that nothing else currently
    // references
    let pml4 = unsafe { &mut *page_ptr.as_mut_ptr::<PageTable>() };

    for index in 256..512 {
        let entry = &mut (*pml4)[index];

        if !entry.is_unused() {
            continue;
        }

        let pdpt_frame = pmm.alloc_frame_zeroed().unwrap_or_else(
            || kernel_panic(
                PanicCode::OutOfMemory,
                "Could not allocate a PDPT while prepopulating the kernel PML4, out of memory"
            )
        );

        entry.set_addr(
            pdpt_frame.as_x86_64(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );
    }
}

/// Verify whether the newly constructed page tables match what the bootloader gave us in critical
/// sections, and panic if they don't to avoid a page fault/triple fault
fn verify_against_bootloader(page_ptr: VirtAddr, sections: &KernelSectionInfo) {
    let old_page_ptr = VirtAddr::from_phys(
        PhysAddr::new(Cr3::read().0.start_address().as_u64())
    );

    let rsp: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp, options(nostack, nomem)) };

    let probes = [
        rsp,
        sections.kernel_start,
        sections.kernel_text_end - 1,
        sections.kernel_rodata_end - 1,
        sections.kernel_end - 1,
        layout::KERNEL_HHDM_BASE.as_u64() + FRAME_SIZE
    ];

    for probe in probes {
        let old = Vmm::translate(old_page_ptr, VirtAddr::new(probe));
        let new = Vmm::translate(page_ptr, VirtAddr::new(probe));

        if old != new {
            kernel_panic(
                PanicCode::InitFailure,
                "New kernel-owned page tables do not match with the bootloader's, which would triple-fault on CR3 switch"
            );
        }
    }
}
