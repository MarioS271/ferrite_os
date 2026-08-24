// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 memory init sequences
//!
//! Authors: MarioS271

use limine::memmap::MEMMAP_BOOTLOADER_RECLAIMABLE;
use limine::request::{MemmapRespData, Response};
use crate::{kdebug, mem, LIMINE_MEMMAP_REQUEST};
use crate::mem::pmm::FRAME_SIZE;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::addr::PhysAddr;
use crate::types::irq_mutex::IrqMutex;
use crate::types::panic_codes::PanicCode;

pub fn mm_init() {
    if let Some(memmap_response) = LIMINE_MEMMAP_REQUEST.response() {
        KSTATE.mm.pmm.call_once(|| IrqMutex::new(mem::pmm::Pmm::init(memmap_response.entries())));
        KSTATE.mm.vmm.call_once(|| IrqMutex::new(mem::vmm::Vmm::init()));

        reclaim_bootloader_memory(memmap_response);
    } else {
        kernel_panic(
            PanicCode::InitFailure,
            "Limine did not provide an initial memmap"
        );
    }

    mem::heap::init();
}

pub fn reclaim_bootloader_memory(memmap_response: &Response<MemmapRespData>) {
    let mut pmm = KSTATE.mm.pmm.get().unwrap().lock();

    #[cfg(feature = "debug-logging")]
    let (mut total_bytes, mut total_entries) = (0u64, 0u64);

    let mut entries: [(u64, u64); 128] = [(0, 0); 128];
    let mut num_entries: usize = 0;

    for entry in memmap_response.entries() {
        if entry.type_ != MEMMAP_BOOTLOADER_RECLAIMABLE {
            continue;
        }

        entries[num_entries] = (entry.base, entry.length);
        num_entries += 1;
    }

    for i in 0..num_entries {
        #[cfg(feature = "debug-logging")]
        {
            total_entries += 1;
            total_bytes += entries[i].1;
        }

        let mut base_addr = entries[i].0;
        let mut remaining = entries[i].1;

        kdebug!("[reclaim] base={:#x} len={:#x}", base_addr, remaining);

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

    kdebug!("reclaimed {} bootloader memory entries ({} MiB)", total_entries, total_bytes / 1024 / 1024);
}
