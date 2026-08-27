// SPDX-License-Identifier: GPL-3.0-only
//! VMM Helpers (x86_64 specific)
//!
//! Authors: MarioS271

use crate::mem::pmm::Pmm;
use crate::mem::vmm::helpers::out_of_memory_panic;
use crate::state::kstate::KSTATE;
use x86_64::structures::paging::{PageTable, PhysFrame};

/// Allocate one physical frame from the PMM, zero it, and return it as a [`PhysFrame`].
///
/// # Panics
/// Panics if the PMM is out of memory.
pub(super) fn alloc_zeroed_frame(pmm: &mut Pmm) -> PhysFrame {
    let hhdm_offset = &KSTATE.mm.hhdm_offset();
    let frame = pmm.alloc_frame().unwrap_or_else(|| out_of_memory_panic());

    // Safe because the PMM gives us a valid piece of memory
    unsafe {
        core::ptr::write_bytes((frame.as_u64() + hhdm_offset) as *mut PageTable, 0x00, 1);
    }

    PhysFrame::from_start_address(x86_64::PhysAddr::new(frame.as_u64())).unwrap()
}
