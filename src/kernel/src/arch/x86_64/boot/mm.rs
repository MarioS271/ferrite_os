// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 memory boot sequence
//!
//! Authors: MarioS271

use crate::arch::x86_64::boot::entry::LIMINE_MEMMAP_REQUEST;
use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::boot_info::KernelSectionInfo;
use crate::mm::pmm::FRAME_SIZE;
use crate::mm::vmm::address_space::AddressSpace;
use crate::mm::vmm::boot_mapping::BootMappings;
use crate::mm::vmm::traits::VmmPaging;
use crate::mm::vmm::Vmm;
use crate::state::kstate::KSTATE;
use crate::{kdebug, mm};
use limine::memmap::MEMMAP_BOOTLOADER_RECLAIMABLE;
use limine::request::{MemmapRespData, Response};
use crate::lib::types::fmt_buffer::FmtBuffer;

pub fn mm_init(section_info: &KernelSectionInfo) {
    let kernel_page: VirtAddr;
    let boot_mappings: BootMappings;
    
    if let Some(memmap_response) = LIMINE_MEMMAP_REQUEST.response() {
        // Safety: init_pmm is only called once which is here; no SMP/threading is currently active
        unsafe { KSTATE.mm.init_pmm(mm::pmm::Pmm::init(memmap_response.entries())) };
        (kernel_page, boot_mappings) = Vmm::setup_kernel_paging(section_info);

        // TODO: reclaim only when we have our own stack
        // reclaim_bootloader_memory(memmap_response);
    } else {
        kernel_panic(
            PanicCode::InitFailure,
            "Limine did not provide an initial memmap"
        );
    }

    mm::heap::init(kernel_page);

    // Safety: init_kernel_addr_space is only called once which is here; no SMP/threading is currently active
    unsafe { KSTATE.mm.init_kernel_addr_space(AddressSpace::empty(kernel_page)) };
    // Safety: kernel_addr_space was initialized one line ago, meaning it is guaranteed to exist
    if let Err(error) = unsafe {
        KSTATE.mm.kernel_addr_space().lock().setup_kernel_vmas(&boot_mappings)
    } {
        use core::fmt::Write;
        
        let mut buffer: FmtBuffer<64> = FmtBuffer::new();
        let _ = write!(buffer, "Failed to set up kernel VMAs ({:?})", error);
        
        kernel_panic(
            PanicCode::InitFailure,
            buffer.as_str()
        );
    }
}

fn reclaim_bootloader_memory(memmap_response: &Response<MemmapRespData>) {
    // Safety: full mm kernel boot happens before this function is called in mm_init, guaranteeing
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
