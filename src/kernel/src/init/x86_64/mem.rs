// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 memory init sequences
//!
//! Authors: MarioS271

use crate::mem::pmm::FRAME_SIZE;
use crate::mem::vmm::address_space::AddressSpace;
use crate::mem::vmm::traits::VmmPaging;
use crate::mem::vmm::Vmm;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::addr::{PhysAddr, VirtAddr};
use crate::types::panic_codes::PanicCode;
use crate::{kdebug, mem, LIMINE_MEMMAP_REQUEST};
use limine::memmap::MEMMAP_BOOTLOADER_RECLAIMABLE;
use limine::request::{MemmapRespData, Response};

pub fn mm_init() {
    let kernel_page: VirtAddr;
    
    if let Some(memmap_response) = LIMINE_MEMMAP_REQUEST.response() {
        // Safety: init_pmm is only called once which is here; no SMP/threading is currently active
        unsafe { KSTATE.mm.init_pmm(mem::pmm::Pmm::init(memmap_response.entries())) };
        kernel_page = Vmm::setup_kernel_page();

        reclaim_bootloader_memory(memmap_response);
    } else {
        kernel_panic(
            PanicCode::InitFailure,
            "Limine did not provide an initial memmap"
        );
    }

    mem::heap::init(kernel_page);

    // Safety: init_kernel_addr_space is only called once which is here; no SMP/threading is currently active
    unsafe { KSTATE.mm.init_kernel_addr_space(AddressSpace::new(kernel_page)) };
    // Safety: kernel_addr_space was initialized one line ago, meaning it is guaranteed to exist
    unsafe { KSTATE.mm.kernel_addr_space().lock().setup_kernel_vmas() };

    remap_kernel_pages();
}

fn reclaim_bootloader_memory(memmap_response: &Response<MemmapRespData>) {
    // Safety: full mm kernel init happens before this function is called in mm_init, guaranteeing
    // that pmm is initialized
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

    #[cfg(feature = "debug-logging")]
    let (mut total_bytes, mut total_entries) = (0u64, 0u64);

    const MAX_ENTRIES: usize = 256;

    let mut entries: [(u64, u64); MAX_ENTRIES] = [(0, 0); MAX_ENTRIES];
    let mut num_entries: usize = 0;

    for entry in memmap_response.entries() {
        if entry.type_ != MEMMAP_BOOTLOADER_RECLAIMABLE {
            continue;
        }

        entries[num_entries] = (entry.base, entry.length);
        num_entries += 1;
    }

    if num_entries >= MAX_ENTRIES {
        kernel_panic(
            PanicCode::InitFailure,
            "Found more reclaimable bootloader entries than currently supported"
        );
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

fn remap_kernel_pages() {
    // Safety: full mm kernel init happens before this function is called in mm_init, guaranteeing
    // that kernel_addr_space is initialized
    let addr_space = unsafe { KSTATE.mm.kernel_addr_space().lock() };
    let kernel_start = &raw const crate::__kernel_start as u64;

    for vma in addr_space.vmas() {
        if vma.start_addr.as_u64() < kernel_start {
            continue;
        }

        let mut addr = vma.start_addr;
        let flags = Vmm::vma_flags_to_page_flags(vma.flags);

        while addr < vma.end_addr {
            // Safety: page_ptr comes from KSTATE.mm.addr_space which was correctly initialized
            // earlier in mm_init; addr comes from a valid VMA
            unsafe {
                Vmm::remap_page(
                    addr_space.page_ptr(),
                    addr,
                    flags
                );
            }

            addr += FRAME_SIZE;
        }
    }
}
