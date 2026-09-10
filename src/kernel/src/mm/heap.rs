// SPDX-License-Identifier: GPL-3.0-only
//! Kernel heap allocator: a fixed-size [`linked_list_allocator::LockedHeap`] set as
//! the `#[global_allocator]`. Does not grow after [`init`].
//!
//! Authors: MarioS271

use crate::arch::x86_64::mm::vmm::page_type::PageType;
use crate::lib::addr::VirtAddr;
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::mm::vmm::{traits::VmmPaging, Vmm};
use crate::state::kstate::KSTATE;
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::PageTableFlags;

// TODO: redo heap allocator properly

/// First virtual address of the heap region (higher-half kernel space).
const HEAP_BASE_ADDRESS: VirtAddr = VirtAddr::new(0xffff_f800_0000_0000);

/// Total heap size in bytes (4 MiB).
const HEAP_SIZE: usize = 0x400_000;

/// The global allocator instance; empty until [`init`].
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Map the heap virtual address range to physical frames and initialize the allocator.
///
/// # Panics
/// Panics with [`PanicCode::OutOfMemory`] if the PMM cannot satisfy any frame allocation.
pub fn init(kernel_root_page: VirtAddr) {
    // Safety: the PMM gets initialized before the heap in mm_init
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

    const HUGE_PAGE_SIZE: usize = 0x200_000; // 2 MiB
    const NUM_HUGE_PAGES: usize = HEAP_SIZE / HUGE_PAGE_SIZE; // 2

    for page in 0..NUM_HUGE_PAGES {
        let phys = pmm.alloc(9).unwrap_or_else(
            || kernel_panic(
                PanicCode::OutOfMemory,
                "Out of memory for heap"
            )
        );

        // Safety: kernel_root_page is the valid kernel PML4, phys is a valid frame that was
        // just mapped
        unsafe {
            if Vmm::map_page(
                &mut pmm,
                kernel_root_page,
                HEAP_BASE_ADDRESS + (page * HUGE_PAGE_SIZE) as u64,
                phys,
                PageType::HugePage2MiB,   // TODO: no arch specific page type here
                PageTableFlags::WRITABLE,
            ).is_err() {
                kernel_panic(
                    PanicCode::OutOfMemory,
                    "Could not map one or more kernel heap pages, out of memory"
                )
            }
        }
    }

    // Safety: boot was never called before, the given heap base addr is valid, static and not
    // used for anything else
    unsafe {
        ALLOCATOR.lock().init(HEAP_BASE_ADDRESS.as_mut_ptr::<u8>(), HEAP_SIZE);
    }
}
