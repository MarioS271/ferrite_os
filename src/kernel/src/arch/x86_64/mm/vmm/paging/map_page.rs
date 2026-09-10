// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `map_page`
//!
//! Authors: MarioS271

use super::super::page_type::PageType;
use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::mm::pmm::Pmm;
use crate::mm::vmm::{VmmError, VmmResult};
use crate::state::kstate::KSTATE;
use x86_64::structures::paging::{PageTable, PageTableFlags, PageTableIndex, PhysFrame};

pub unsafe fn map_page(
    pmm: &mut Pmm,
    page_ptr: VirtAddr,
    virt: VirtAddr,
    phys: PhysAddr,
    page_type: PageType,
    flags: PageTableFlags,
) -> VmmResult {
    #[cfg(feature = "debug-checks")]
    {
        use super::helpers::misaligned_address_panic;
        let align = &(page_type as u64);
        if phys.as_u64() % align != 0 || virt.as_u64() % align != 0 {
            misaligned_address_panic(page_type);
        }
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
            let frame = pmm.alloc_frame_zeroed().ok_or(VmmError::OutOfMemory)?;
            entry.set_frame(
                PhysFrame::containing_address(frame.as_x86_64()),
                intermediate_flags,
            );
        }

        current_pagetable = (entry.frame().unwrap().start_address().as_u64() + hhdm_offset) as *mut PageTable;
    }

    // Safe: current_pagetable is a valid
    let entry = unsafe {
        &mut current_pagetable.as_mut().unwrap()[match page_type {
            PageType::Normal => virt.p1_index(),
            PageType::HugePage2MiB => virt.p2_index(),
            PageType::HugePage1GiB => virt.p3_index(),
        }]
    };

    if page_type == PageType::Normal {
        entry.set_frame(PhysFrame::containing_address(phys.as_x86_64()), flags | PageTableFlags::PRESENT);
    } else {
        entry.set_addr(phys.as_x86_64(), flags | PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE);
    }

    #[cfg(feature = "vmm-debug-logging")]
    crate::kdebug!("[VMM] map new page at virt {virt:#x}, phys {phys:#x}, type {page_type}, flags {flags:?}");

    Ok(())
}
