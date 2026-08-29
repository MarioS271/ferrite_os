// SPDX-License-Identifier: GPL-3.0-only
//! Memory management subcategory placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::mem::pmm::Pmm;
use crate::mem::vmm::address_space::AddressSpace;
use crate::panic::kernel_panic;
use crate::types::irq_mutex::IrqMutex;
use crate::types::panic_codes::PanicCode;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU64, Ordering};

/// Holds memory management state such as slab allocators and virtual memory areas.
pub struct Mm {
    hhdm_offset: AtomicU64,
    pmm: UnsafeCell<MaybeUninit<IrqMutex<Pmm>>>,
    kernel_addr_space: UnsafeCell<MaybeUninit<IrqMutex<AddressSpace>>>
}

/// Safety: Sync is safe because everything is only initialized once and non-sync objects
/// are wrapped in IrqMutex
unsafe impl Sync for Mm {}

impl Mm {
    /// Constructor; initializes all values zeroed or uninited
    pub const fn new() -> Self {
        Self {
            hhdm_offset: AtomicU64::new(0),
            pmm: UnsafeCell::new(MaybeUninit::uninit()),
            kernel_addr_space: UnsafeCell::new(MaybeUninit::uninit())
        }
    }

    /// Sets the hhdm offset
    pub fn set_hhdm_offset(&self, offset: u64) {
        self.hhdm_offset.store(offset, Ordering::Release);
    }

    /// Move the given [`Pmm`] into `KSTATE::mm::pmm`
    pub fn init_pmm(&self, pmm: Pmm) {
        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.pmm.get()).write(IrqMutex::new(pmm)); }
    }

    /// Move the given [`AddressSpace`] into `KSTATE::mm::kernel_addr_space`
    pub fn init_kernel_addr_space(&self, kernel_addr_space: AddressSpace) {
        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.kernel_addr_space.get()).write(IrqMutex::new(kernel_addr_space)); }
    }

    /// Getter for the hhdm offset
    pub fn hhdm_offset(&self) -> u64 {
        let hhdm = self.hhdm_offset.load(Ordering::Acquire);
        if hhdm != 0 {
            hhdm
        } else {
            kernel_panic(
                PanicCode::InitFailure,
                "Attempted to access KSTATE.mm.hhdm_offset before it was initialized"
            )
        }
    }

    /// Getter for KSTATE::mm::pmm
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    pub fn pmm(&self) -> &IrqMutex<Pmm> {
        // Safety:
        // 1) the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        // 2) assume_init_ref is not guaranteed to be safe, the caller must guarantee this
        unsafe { (*self.pmm.get()).assume_init_ref() }
    }

    /// Getter for KSTATE::mm::kernel_address_space
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    pub fn kernel_addr_space(&self) -> &IrqMutex<AddressSpace> {
        // Safety:
        // 1) the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        // 2) assume_init_ref is not guaranteed to be safe, the caller must guarantee this
        unsafe { (*self.kernel_addr_space.get()).assume_init_ref() }
    }
}
