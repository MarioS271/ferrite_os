// SPDX-License-Identifier: GPL-3.0-only
//! Physical Memory Manager (PMM): buddy allocator that tracks and hands out physical frames.
//!
//! Authors: MarioS271

use crate::{kdebug, kinfo};
use x86_64::PhysAddr;
use limine::memmap;

pub const FRAME_SIZE: u64 = 4096;
const MAX_ORDER: usize = 10;
const NUM_ORDERS: usize = MAX_ORDER + 1;

/// Buddy-Allocator based tracker of free and used physical frames.
pub struct Pmm {
    free: [Option<PhysAddr>; 11],
    hhdm_offset: u64
}

impl Pmm {
    /// Initialize the PMM from the Limine memory map, coalescing usable frames into the buddy free lists.
    pub fn init(entries: &[&memmap::Entry], hhdm_offset: u64) -> Self {
        let mut pmm = Pmm {
            free: [None; NUM_ORDERS],
            hhdm_offset: hhdm_offset
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

        for &entry in entries {
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

        pmm
    }

    pub fn alloc(&mut self, order: usize) -> Option<PhysAddr> {
        if order > 10 { return None; }

        if self.free[order] != None {
            return self.pop(order);
        }

        let mut current: usize = MAX_ORDER;
        for o in order..=MAX_ORDER {
            if self.free[o] != None {
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

    pub fn alloc_frame(&mut self) -> Option<PhysAddr> {
        self.alloc(0)
    }

    pub fn free(&mut self, mut addr: PhysAddr, mut order: usize) {
        while order < MAX_ORDER {
            let buddy = PhysAddr::new(addr.as_u64() ^ (FRAME_SIZE << order));

            let mut current = self.free[order];
            let mut prev_ptr = &mut self.free[order] as *mut Option<PhysAddr>;
            let mut found = false;

            while let Some(node) = current {
                if node == buddy {
                    // Safe: node is a valid free block; first 8 bytes hold the next pointer written by push()
                    let next = unsafe {
                        let val = core::ptr::read((node.as_u64() + self.hhdm_offset) as *const u64);
                        if val == 0 { None } else { Some(PhysAddr::new(val)) }
                    };
                    // Safe: prev_ptr points to either free[order] or a next-pointer field in a free block, both valid via &mut self
                    unsafe { *prev_ptr = next; }
                    found = true;
                    break;
                }

                prev_ptr = (node.as_u64() + self.hhdm_offset) as *mut Option<PhysAddr>;

                // Safe: HHDM maps all usable memory; next pointer was written by push(), 0 = end of list
                current = unsafe {
                    let val = core::ptr::read((node.as_u64() + self.hhdm_offset) as *const u64);
                    if val == 0 { None } else { Some(PhysAddr::new(val)) }
                };
            }

            if !found { break; }

            addr = PhysAddr::new(addr.as_u64().min(buddy.as_u64()));
            order += 1;
        }

        self.push(order, addr);
    }

    pub fn free_frame(&mut self, addr: PhysAddr) {
        self.free(addr, 0)
    }


    /// Remove and return the head block from the order-`order` free list, or `None` if empty.
    fn pop(&mut self, order: usize) -> Option<PhysAddr> {
        let head = self.free[order]?;
        let result;

        unsafe {
            // Safe: head was placed here by push() with a valid usable physical frame
            // HHDM maps all usable memory, so head + hhdm_offset is a valid mapped address
            result = core::ptr::read(
                (head.as_u64() + self.hhdm_offset) as *const u64
            );
        }
        if result == 0 {
            self.free[order] = None;
        } else {
            self.free[order] = Some(PhysAddr::new(result));
        }

        Some(head)
    }

    /// Prepend `addr` onto the order-`order` free list.
    fn push(&mut self, order: usize, addr: PhysAddr) {
        unsafe {
            // Safe: addr is a valid usable physical frame sourced from the Limine memmap
            // HHDM maps all usable memory, so addr + hhdm_offset is a valid mapped address
            core::ptr::write(
                (addr.as_u64() + self.hhdm_offset) as *mut u64,
                 self.free[order].map_or(0, |addr| addr.as_u64())
            );
        }
        self.free[order] = Some(addr);
    }
}
