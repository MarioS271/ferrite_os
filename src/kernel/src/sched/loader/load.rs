// SPDX-License-Identifier: GPL-3.0-only
//! ELF loading logic
//!
//! Authors: MarioS271

use crate::kemerg;
use crate::lib::addr::VirtAddr;
use crate::mm::pmm::FRAME_SIZE;
use crate::mm::vmm::address_space::AddressSpace;
use crate::mm::vmm::traits::VmmPaging;
use crate::mm::vmm::vma::VmaFlags;
use crate::mm::vmm::Vmm;
use crate::sched::loader::defs::phdrs::ElfPhdr;
use crate::state::kstate::KSTATE;

const USER_STACK_VIRT: VirtAddr = VirtAddr::new(0x0000_7fff_ffff_0000);

/// Map VMAs and pages according to the ELF phdrs and copy the ELF binary into the
/// freshly mapped memory
pub fn map_phdrs_and_copy_elf(addr_space: &mut AddressSpace, phdrs: &[ElfPhdr], elf: &[u8]) {
    for phdr in phdrs {
        if phdr.p_type != ElfPhdr::PT_LOAD {
            continue;
        }

        let vaddr_start = VirtAddr::new(phdr.p_vaddr).align_down(FRAME_SIZE);
        let vaddr_end = VirtAddr::new(phdr.p_vaddr + phdr.p_memsz).align_up(FRAME_SIZE);
        let size = vaddr_end.as_u64() - vaddr_start.as_u64();

        let mut vma_flags = VmaFlags::USER;
        if phdr.p_flags & ElfPhdr::PF_R != 0 { vma_flags |= VmaFlags::READ; }
        if phdr.p_flags & ElfPhdr::PF_W != 0 { vma_flags |= VmaFlags::WRITE; }
        if phdr.p_flags & ElfPhdr::PF_X != 0 { vma_flags |= VmaFlags::EXEC; }

        // Safety: the PMM was already initialized in stage 2
        let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

        let res = Vmm::map_region(
            &mut pmm,
            addr_space,
            vaddr_start,
            size,
            vma_flags
        );

        drop(pmm);

        if let Err(error) = res {
            kemerg!("Vmm::map_region failed, halting (error code: {:?})", error);
            crate::cpu::instructions::halt_forever();
        }

        // Safety: map_region correctly mapped memory in this code path and nothing is running
        // in parallel
        unsafe { copy_segment_data(addr_space, phdr, elf) };
    }
}

/// Copies an ELF segment into the process's address space
///
/// # Safety
/// The caller must ensure all the following things:
/// - `p_vaddr` to `p_vaddr + p_filesz` needs to be fully mapped in `addr_space`
/// - No references/pointers to the target frames should exist and no other CPU should be
///   able to concurrently operate on this data
unsafe fn copy_segment_data(addr_space: &AddressSpace, phdr: &ElfPhdr, elf: &[u8]) {
    let mut copied = 0u64;

    while copied < phdr.p_filesz {
        let dst_vaddr = phdr.p_vaddr + copied;
        let page_base = VirtAddr::new(dst_vaddr).align_down(FRAME_SIZE);
        let offset_in_page = dst_vaddr - page_base.as_u64();

        let phys = Vmm::translate(addr_space.page_ptr(), page_base).unwrap();
        let remaining_space_in_page = FRAME_SIZE - offset_in_page;
        let remaining_to_copy = phdr.p_filesz - copied;
        let chunk = remaining_space_in_page.min(remaining_to_copy);

        unsafe {
            let src = elf.as_ptr().add((phdr.p_offset + copied) as usize);
            core::ptr::copy_nonoverlapping(
                src,
                phys.as_mut_hhdm_ptr::<u8>().add(offset_in_page as usize),
                chunk as usize
            );
        }

        copied += chunk;
    }
}

/// Set up a stack for the user process
pub fn setup_user_stack(addr_space: &mut AddressSpace) -> VirtAddr {
    let stack_size = FRAME_SIZE * 4;    // TODO: dynamically do this
    let stack_top = USER_STACK_VIRT;
    let stack_bottom = stack_top - stack_size;

    // Safety: the PMM was already initialized in stage 2
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

    // TODO: lazily map this (needs user pf handler)
    let res = Vmm::map_region(
        &mut pmm,
        addr_space,
        stack_bottom,
        stack_size,
        VmaFlags::USER | VmaFlags::READ | VmaFlags::WRITE
    );

    drop(pmm);

    if let Err(error) = res {
        kemerg!("Vmm::map_region failed, halting (error code: {:?})", error);
        crate::cpu::instructions::halt_forever();
    }

    stack_top
}
