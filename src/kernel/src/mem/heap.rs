//! Kernel heap allocator.
//!
//! Uses [`linked_list_allocator::LockedHeap`] as the `#[global_allocator]`, which
//! enables `alloc` crate types (`Box`, `Vec`, `String`, etc.) anywhere in the
//! kernel after [`init`] is called.
//!
//! The heap is backed by a contiguous range of virtual pages starting at
//! `HEAP_BASE_ADDRESS`. Physical frames for those pages are allocated from the PMM
//! at startup and mapped writable — the heap does not dynamically grow.
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

/// First virtual address of the heap region. Chosen to be in the higher-half
/// kernel address space, well above the HHDM and kernel image regions.
static HEAP_BASE_ADDRESS: VirtAddr = VirtAddr::new(0xffff_8080_0000_0000);

/// Total heap size in bytes (4 MiB). All physical frames are allocated and
/// mapped during [`init`]; the heap does not grow after that.
static HEAP_SIZE: usize = 0x400_000;

/// The global allocator instance. Starts empty; [`init`] passes the heap region
/// to `ALLOCATOR.lock().init(...)` to make it functional.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Map the heap virtual address range to physical frames and initialize the allocator.
///
/// Iterates over `HEAP_SIZE / FRAME_SIZE` pages, allocates one physical frame per
/// page from the PMM, and maps each page at `HEAP_BASE_ADDRESS + page_index * FRAME_SIZE`
/// with `PRESENT | WRITABLE` flags. After all pages are mapped, calls
/// `ALLOCATOR.lock().init(...)` to inform `linked_list_allocator` of the usable region.
///
/// # Panics
/// Panics with [`PanicCode::OutOfMemory`] if the PMM cannot satisfy any frame
/// allocation during setup.
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
