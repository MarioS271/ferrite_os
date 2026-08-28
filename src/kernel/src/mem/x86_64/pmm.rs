// SPDX-License-Identifier: GPL-3.0-only
//! Physical Memory Manager (PMM): buddy allocator that tracks and hands out physical frames.
//!
//! Authors: MarioS271

use crate::{kdebug, kinfo};
use limine::memmap;
use crate::state::kstate::KSTATE;
use crate::types::addr::PhysAddr;

pub const FRAME_SIZE: u64 = 4096;
const MAX_ORDER: usize = 10;
const NUM_ORDERS: usize = MAX_ORDER + 1;

/// Buddy-Allocator based tracker of free and head physical frames.
pub struct Pmm {
    head: [Option<PhysAddr>; 11],
    total_mem: u64
}

impl Pmm {
    /// Initialize the PMM from the Limine memory map, coalescing usable frames into the buddy free lists.
    pub fn init(entries: &[&memmap::Entry]) -> Self {
        let mut pmm = Pmm {
            head: [None; NUM_ORDERS],
            total_mem: 0
        };

        #[cfg(feature = "debug-logging")]
        {
            let mut total_usable = 0u64;
            let mut total_reclaimable = 0u64;
            let mut count_usable = 0u64;
            let mut count_reclaimable = 0u64;
            let mut count_reserved = 0u64;
            let mut count_acpi = 0u64;

            for &entry in entries {
                match entry.type_ {
                    memmap::MEMMAP_USABLE => { count_usable += 1; total_usable += entry.length; }
                    memmap::MEMMAP_BOOTLOADER_RECLAIMABLE => { count_reclaimable += 1; total_reclaimable += entry.length; }
                    memmap::MEMMAP_RESERVED => { count_reserved += 1; }
                    memmap::MEMMAP_ACPI_RECLAIMABLE => { count_acpi += 1; }
                    _ => {}
                }
            }

            kdebug!("[PMM] total entries: {}", entries.len());
            kdebug!("[PMM] usable: {} regions, {} MiB", count_usable, total_usable / 1024 / 1024);
            kdebug!("[PMM] reclaimable: {} regions, {} MiB", count_reclaimable, total_reclaimable / 1024 / 1024);
            kdebug!("[PMM] reserved: {} regions", count_reserved);
            kdebug!("[PMM] acpi: {} regions", count_acpi);
        }

        let mut total_mem = 0u64;
        for &entry in entries {
            total_mem = total_mem.max(entry.base + entry.length);

            if entry.type_ != memmap::MEMMAP_USABLE { continue; }

            let mut addr = entry.base;
            let mut remaining = entry.length;

            if addr == 0 {
                addr += FRAME_SIZE;
                remaining -= FRAME_SIZE;
            }

            while remaining >= FRAME_SIZE {
                let mut order = MAX_ORDER;
                while order > 0 {
                    let block_size = FRAME_SIZE << order;
                    if remaining >= block_size && addr % (FRAME_SIZE << order) == 0 { break; }
                    order -= 1;
                }

                pmm.free(PhysAddr::new(addr), order);

                let block_size = FRAME_SIZE << order;
                addr += block_size;
                remaining -= block_size;
            }
        }

        kinfo!("Initialized PMM");

        pmm.total_mem = total_mem;
        pmm
    }

    /// Getter for the length of total memory, returns the highest physical address given by the memmap
    pub fn get_total_mem(&self) -> u64 {
        self.total_mem
    }

    /// Allocate a block of `FRAME_SIZE << order` bytes
    pub fn alloc(&mut self, order: usize) -> Option<PhysAddr> {
        if order > 10 { return None; }

        if self.head[order] != None {
            return self.pop(order);
        }

        let mut current: usize = MAX_ORDER;
        for o in order..=MAX_ORDER {
            if self.head[o] != None {
                current = o;
                break;
            }
            if o == MAX_ORDER {
                return None;
            }
        }

        let addr = self.pop(current)?;
        while current > order {
            current -= 1;
            let buddy = PhysAddr::new(addr.as_u64() + (FRAME_SIZE << current));
            self.push(current, buddy);
        }

        Some(addr)
    }

    /// Wrapper for [`alloc(0)`](Self::alloc), allocates exactly one frame (order 0)
    pub fn alloc_frame(&mut self) -> Option<PhysAddr> {
        self.alloc(0)
    }

    /// Return a block of `FRAME_SIZE << order` bytes to the free lists, merging with available buddies
    pub fn free(&mut self, mut addr: PhysAddr, mut order: usize) {
        'outer: while order < MAX_ORDER {
            if self.head[order].is_none() {
                break;
            }

            let buddy = PhysAddr::new(addr.as_u64() ^ (FRAME_SIZE << order));
            let mut current_node_addr = self.head[order].unwrap();
            let mut prev_node_addr = PhysAddr::null();

            loop {
                let next_node_addr: u64 = unsafe { core::ptr::read(current_node_addr.as_hhdm_ptr()) };

                if current_node_addr == buddy {
                    if prev_node_addr.as_u64() == 0 {
                        self.head[order] = Some(PhysAddr::new(next_node_addr));
                    }
                    else {
                        unsafe { core::ptr::write(prev_node_addr.as_mut_hhdm_ptr(), next_node_addr) };
                    }
                    break;
                }

                if next_node_addr == 0 {
                    break 'outer;
                }
                prev_node_addr = current_node_addr;
                current_node_addr = PhysAddr::new(next_node_addr);
            }

            order += 1;
            addr = PhysAddr::new(core::cmp::min(addr.as_u64(), buddy.as_u64()));
        }

        self.push(order, addr);
    }

    /// Wrapper for [`free(addr, 0)`](Self::free), frees exactly one frame (order 0)
    pub fn free_frame(&mut self, addr: PhysAddr) {
        self.free(addr, 0)
    }


    /// Remove and return the head block from the order-`order` free list, or `None` if empty.
    fn pop(&mut self, order: usize) -> Option<PhysAddr> {
        let hhdm_offset = &KSTATE.mm.hhdm_offset();
        let head = self.head[order]?;
        let result;

        unsafe {
            // Safe: head was placed here by push() with a valid usable physical frame
            // HHDM maps all usable memory, so head + hhdm_offset is a valid mapped address
            result = core::ptr::read(
                (head.as_u64() + hhdm_offset) as *const u64
            );
        }
        if result == 0 {
            self.head[order] = None;
        } else {
            self.head[order] = Some(PhysAddr::new(result));
        }

        Some(head)
    }

    /// Prepend `addr` onto the order-`order` free list.
    fn push(&mut self, order: usize, addr: PhysAddr) {
        let hhdm_offset = &KSTATE.mm.hhdm_offset();

        // Safe: addr is a valid usable physical frame sourced from the Limine memmap
        // HHDM maps all usable memory, so addr + hhdm_offset is a valid mapped address
        unsafe {
            core::ptr::write(
                (addr.as_u64() + hhdm_offset) as *mut u64,
                 self.head[order].map_or(0, |addr| addr.as_u64())
            );
        }

        self.head[order] = Some(addr);
    }
}
