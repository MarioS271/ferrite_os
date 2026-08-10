// SPDX-License-Identifier: GPL-3.0-only
//! Kernel heap allocator: a fixed-size [`linked_list_allocator::LockedHeap`] set as
//! the `#[global_allocator]`. Does not grow after [`init`].
//!
//! Authors: MarioS271

use crate::mem::x86_64::pmm::{Pmm, FRAME_SIZE};
use crate::mem::vmm::Vmm;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;
use linked_list_allocator::LockedHeap;
use crate::types::irq_mutex::IrqMutex;

/// First virtual address of the heap region (higher-half kernel space).
static HEAP_BASE_ADDRESS: VirtAddr = VirtAddr::new(0xffff_8080_0000_0000);

/// Total heap size in bytes (4 MiB).
static HEAP_SIZE: usize = 0x400_000;

/// The global allocator instance; empty until [`init`].
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Map the heap virtual address range to physical frames and initialize the allocator.
///
/// # Panics
/// Panics with [`PanicCode::OutOfMemory`] if the PMM cannot satisfy any frame allocation.
pub fn init(pmm_mutex: &IrqMutex<Pmm>, vmm_mutex: &IrqMutex<Vmm>) {
    let mut pmm = pmm_mutex.lock();
    let mut vmm = vmm_mutex.lock();

    unsafe {
        for page in 0..(HEAP_SIZE / FRAME_SIZE as usize) {
            let phys_page = pmm.alloc_frame().unwrap_or_else(
                || kernel_panic(
                    PanicCode::OutOfMemory,
                    "Out of memory for heap",
                )
            );
            vmm.map_page(
                &mut pmm,
                HEAP_BASE_ADDRESS + (page * FRAME_SIZE as usize) as u64,
                phys_page,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );
        }

        ALLOCATOR.lock().init(HEAP_BASE_ADDRESS.as_mut_ptr::<u8>(), HEAP_SIZE);
    }
}
