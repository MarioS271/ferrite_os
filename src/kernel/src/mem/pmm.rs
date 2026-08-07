// SPDX-License-Identifier: GPL-3.0-only
//! Physical Memory Manager (PMM): buddy allocator that tracks and hands out physical frames.
//!
//! Authors: MarioS271

use crate::{kinfo};
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

// TODO: Pmm contains no raw pointers; verify PhysAddr: Send + Sync (it is a u64 newtype),
// then remove these manual impls and let the compiler derive them.
unsafe impl Send for Pmm {}
unsafe impl Sync for Pmm {}

impl Pmm {
    /// Initialize the PMM from the Limine memory map, coalescing usable frames into the buddy free lists.
    pub fn init(entries: &[&memmap::Entry], hhdm_offset: u64) -> Self {
        let mut pmm = Pmm {
            free: [None; NUM_ORDERS],
            hhdm_offset: hhdm_offset
        };

        for &entry in entries {
            if entry.type_ != memmap::MEMMAP_USABLE { continue; }

            let first_frame = entry.base / FRAME_SIZE;
            let frame_count = entry.length / FRAME_SIZE;

            for frame in first_frame..(first_frame + frame_count) {
                if frame == 0 { continue; }
                pmm.free_frame(PhysAddr::new(frame * FRAME_SIZE));
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

        let mut addr = self.pop(current)?;
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

                // Safe: node is a valid free block; casting its next-pointer field to *mut Option<PhysAddr> is valid (same u64 layout)
                prev_ptr = unsafe { (node.as_u64() + self.hhdm_offset) as *mut Option<PhysAddr> };

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
