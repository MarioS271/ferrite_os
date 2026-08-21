// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 memory init sequences
//!
//! Authors: MarioS271

use limine::memmap::MEMMAP_BOOTLOADER_RECLAIMABLE;
use limine::request::{MemmapRespData, Response};
use x86_64::PhysAddr;
use crate::mem::pmm::FRAME_SIZE;
use crate::state::kstate::KSTATE;

pub fn reclaim_bootloader_memory(memmap_response: &Response<MemmapRespData>) {
    let mut pmm = KSTATE.mm.pmm.get().unwrap().lock();

    for entry in memmap_response.entries() {
        if entry.type_ != MEMMAP_BOOTLOADER_RECLAIMABLE {
            continue;
        }

        let mut base_addr = entry.base;
        let mut remaining = entry.length;

        while remaining >= FRAME_SIZE {
            let mut order = 10usize;
            while order > 0 {
                let block_size = FRAME_SIZE << order;
                if base_addr % block_size == 0 && remaining >= block_size {
                    break;
                }
                order -= 1;
            }

            pmm.free(PhysAddr::new(base_addr), order);

            let block_size = FRAME_SIZE << order;
            base_addr += block_size;
            remaining -= block_size;
        }
    }
}
