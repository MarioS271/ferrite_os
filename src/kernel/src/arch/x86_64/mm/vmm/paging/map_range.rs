// SPDX-License-Identifier: GPL-3.0-only
//! VMM Paging (x86_64): `map_range`
//!
//! Authors: MarioS271

use crate::arch::x86_64::mm::vmm::page_type::{PageType, HUGE_PAGE_SIZE_1GIB, HUGE_PAGE_SIZE_2MIB};
use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::mm::pmm::Pmm;
use crate::mm::vmm::VmmResult;
use x86_64::structures::paging::PageTableFlags;

pub unsafe fn map_range(
    pmm: &mut Pmm,
    page_ptr: VirtAddr,
    virt: VirtAddr,
    phys: PhysAddr,
    size: u64,
    flags: PageTableFlags,
    can_overmap: bool
) -> VmmResult {
    #[cfg(feature = "debug-checks")]
    {
        use super::helpers::misaligned_address_panic;
        use crate::mm::pmm::FRAME_SIZE;
        if !virt.is_aligned(FRAME_SIZE) || !phys.is_aligned(FRAME_SIZE) || size % FRAME_SIZE != 0 {
            misaligned_address_panic(PageType::Normal);
        }
    }

    let mut offset = 0u64;

    while offset < size {
        let virt = virt + offset;
        let phys = phys + offset;
        let remaining = size - offset;

        let fits = |page_size: u64| {
            virt.is_aligned(page_size)
                && phys.is_aligned(page_size)
                && (can_overmap || remaining >= page_size)
        };

        let page_type = if fits(HUGE_PAGE_SIZE_1GIB) {
            PageType::HugePage1GiB
        }
        else if fits(HUGE_PAGE_SIZE_2MIB) {
            PageType::HugePage2MiB
        }
        else {
            PageType::Normal
        };

        unsafe {
            super::map_page::map_page(pmm, page_ptr, virt, phys, page_type, flags)?;
        }

        offset += page_type as u64;
    }

    Ok(())
}
