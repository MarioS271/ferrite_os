//! mem/heap.rs
//! Heap Allocator
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;

static HEAP_BASE_ADDRESS: VirtAddr = VirtAddr::new(0xffff_8080_0000_0000);
static HEAP_SIZE: usize = 0x400_000;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

