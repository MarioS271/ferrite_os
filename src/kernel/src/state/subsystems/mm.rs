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

/// Safety:
/// - `hhdm_offset` is an [`AtomicU64`] and therefore is already [`Sync`]
/// - `pmm` and `kernel_addr_space` are only written to exactly once before any concurrent
///   access can occur. After the calls to `init_pmm`/`init_kernel_addr_space`, no mutable
///   reference or pointer to any of the two is ever held again
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
    ///
    /// # Safety
    /// This method derefs `self.pmm.get()`
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - There must be no mutable references or pointers to `self.pmm`
    /// - This method must be called exactly once
    pub unsafe fn init_pmm(&self, pmm: Pmm) {
        unsafe { (*self.pmm.get()).write(IrqMutex::new(pmm)); }
    }

    /// Move the given [`AddressSpace`] into `KSTATE::mm::kernel_addr_space`
    ///
    /// # Safety
    /// This method derefs `self.kernel_addr_space.get()`.
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - There must be no mutable references or pointers to `self.kernel_addr_space`
    /// - This method must be called exactly once
    pub unsafe fn init_kernel_addr_space(&self, kernel_addr_space: AddressSpace) {
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

    /// Getter for `KSTATE::mm::pmm`
    ///
    /// # Safety
    /// This method derefs `self.pmm.get()` and calls [`MaybeUninit::assume_init_ref()`].
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - `self.pmm` must have been initialized first (via `init_pmm`)
    /// - There must be no mutable references or pointers to `self.pmm`
    pub unsafe fn pmm(&self) -> &IrqMutex<Pmm> {
        unsafe { (*self.pmm.get()).assume_init_ref() }
    }

    /// Getter for `KSTATE::mm::kernel_address_space`
    ///
    /// # Safety
    /// This method derefs `self.kernel_addr_space.get()` and calls [`MaybeUninit::assume_init_ref()`].
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - `self.kernel_addr_space` must have been initialized first (via `init_kernel_addr_space`)
    /// - There must be no mutable references or pointers to `self.kernel_addr_space`
    pub unsafe fn kernel_addr_space(&self) -> &IrqMutex<AddressSpace> {
        unsafe { (*self.kernel_addr_space.get()).assume_init_ref() }
    }
}
