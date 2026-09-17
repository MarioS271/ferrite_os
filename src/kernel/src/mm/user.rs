// SPDX-License-Identifier: GPL-3.0-only
//! High-Level MM functions that are used for allocating user-space memory
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::mm::layout;
use crate::mm::mm_error::{MmError, MmResult};
use crate::mm::pmm::FRAME_SIZE;
use crate::mm::vmm::Vmm;
use crate::mm::vmm::address_space::AddressSpace;
use crate::mm::vmm::vma::VmaFlags;
use crate::state::kstate::KSTATE;

/// Lazily allocate memory in the given user process's address space
///
/// By passing the `fixed_addr` parameter, the caller can request the memory to be mapped into a
/// specific location in the address space
pub fn alloc(
    addr_space: &mut AddressSpace,
    fixed_addr: Option<VirtAddr>,
    size: u64,
    flags: VmaFlags
) -> MmResult<VirtAddr> {
    let size = VirtAddr::new(size).align_up(FRAME_SIZE).as_u64();

    let virt = match fixed_addr {
        Some(addr) => {
            if !addr.is_aligned(FRAME_SIZE) {
                return Err(MmError::MisalignedAddress);
            }
            addr
        },
        None => {
            addr_space
                .find_free_range(layout::USER_MMAP_BASE, layout::USER_STACK_TOP, size)
                .ok_or(MmError::OutOfMemory)?
        }
    };

    Vmm::lazy_map_region(addr_space, virt, size, flags | VmaFlags::USER)?;

    Ok(virt)
}

/// Free previously by [`alloc`] allocated memory and release all associated frames back into
/// the buddy allocator
///
/// # Safety
/// The caller must ensure that no live code or data holds references into any address mapped
/// within `[virt, vma.end_addr)` after this method call returns
pub unsafe fn free(
    addr_space: &mut AddressSpace,
    virt: VirtAddr
) -> MmResult<()> {
    // Safety: the PMM is initialized in stage 2
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };
    unsafe { Vmm::unmap_region(&mut pmm, addr_space, virt)? };
    Ok(())
}

/// Change an already mapped memory region's access flags
///
/// # Safety
/// The caller must ensure that for example when removing the `WRITE` flag,
/// no mutable references to the to be modified memory is held.
pub unsafe fn remap_user_mem(
    addr_space: &mut AddressSpace,
    virt: VirtAddr,
    new_flags: VmaFlags
) -> MmResult<()> {
    unsafe { Vmm::remap_region(addr_space, virt, new_flags | VmaFlags::USER)? };
    Ok(())
}
