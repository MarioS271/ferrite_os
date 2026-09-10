// SPDX-License-Identifier: GPL-3.0-only
//! High-Level Memory Management Functions
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::mm::layout;
use crate::mm::pmm::FRAME_SIZE;
use crate::mm::vmm::address_space::AddressSpace;
use crate::mm::vmm::vma::VmaFlags;
use crate::mm::vmm::{Vmm, VmmError};
use crate::state::kstate::KSTATE;

/// A wrapper around `Result<T, MmError>`
pub type MmResult<T> = Result<T, MmError>;

/// An enum which describes possible memory management errors that can occur
#[derive(Debug)]
pub enum MmError {
    /// No free memory left to allocate
    OutOfMemory,
    /// The given address is not frame aligned
    MisalignedAddress,
    /// The requested region overlaps with an existing one
    Overlap,
    /// No region at the given address found
    NotFound
}
impl From<VmmError> for MmError {
    /// Convert a given [`VmmError`] to a [`MmError`]
    fn from(value: VmmError) -> Self {
        match value {
            VmmError::OutOfMemory => Self::OutOfMemory,
            VmmError::VmaOverlap => Self::Overlap,
            VmmError::VmaNotFound => Self::NotFound,
            VmmError::InvalidUnmap | VmmError::InvalidRemap => Self::NotFound
        }
    }
}

/// Lazily allocate memory in the given user process's address space
///
/// By passing the `fixed_addr` parameter, the caller can request the memory to be mapped into a
/// specific location in the address space
pub fn alloc_user_mem(
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

/// Free previously by [`alloc_user_mem`] allocated memory and release all associated frames back into
/// the buddy allocator
///
/// # Safety
/// The caller must ensure that no live code or data holds references into any address mapped
/// within `[virt, vma.end_addr)` after this method call returns
pub unsafe fn free_user_mem(
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
