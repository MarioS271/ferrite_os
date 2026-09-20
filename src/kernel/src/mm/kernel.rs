// SPDX-License-Identifier: GPL-3.0-only
//! High-Level kernel memory management functions
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::mm::mm_error::MmResult;
use crate::mm::pmm::{FRAME_SIZE, Pmm};
use crate::mm::vmm::Vmm;
use crate::mm::vmm::address_space::AddressSpace;
use crate::mm::vmm::vma::VmaFlags;
use crate::state::kstate::KSTATE;

/// Eagerly allocate a fixed-size 16 KiB kernel stack
///
/// Returns a virtual pointer to the top of the newly allocated stack
///
/// # Safety
/// The caller must guarantee that boot stage 2 has already been completed
pub unsafe fn alloc_stack(
    pmm: &mut Pmm,
    addr_space: &mut AddressSpace
) -> MmResult<VirtAddr> {
    let slot = KSTATE.mm.alloc_kernel_stack_slot();

    let stack_bottom = slot + FRAME_SIZE;
    let stack_top = stack_bottom + super::layout::KERNEL_STACK_SIZE;

    Vmm::map_region(
        pmm,
        addr_space,
        stack_bottom,
        super::layout::KERNEL_STACK_SIZE,
        VmaFlags::READ | VmaFlags::WRITE
    )?;

    Ok(stack_top)
}
