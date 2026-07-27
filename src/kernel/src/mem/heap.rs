//! mem/heap.rs
//! Heap Allocator
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::mem::pmm;
use crate::mem::vmm;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;
use linked_list_allocator::LockedHeap;

static HEAP_BASE_ADDRESS: VirtAddr = VirtAddr::new(0xffff_8080_0000_0000);
static HEAP_SIZE: usize = 0x400_000;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    unsafe {
        for page in 0..(HEAP_SIZE / pmm::FRAME_SIZE as usize) {
                let phys_page = pmm::alloc().unwrap_or_else(
                    || kernel_panic(
                        PanicCode::OutOfMemory,
                        "Out of memory for heap",
                    )
                );
                vmm::map_page(
                    HEAP_BASE_ADDRESS + (page * pmm::FRAME_SIZE as usize) as u64,
                    phys_page,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                );
        }

        ALLOCATOR.lock().init(HEAP_BASE_ADDRESS.as_mut_ptr::<u8>(), HEAP_SIZE);
    }
}
