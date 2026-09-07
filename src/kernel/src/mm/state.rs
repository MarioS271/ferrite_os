// SPDX-License-Identifier: GPL-3.0-only
//! Memory management subcategory for [`KState`]
//!
//! Authors: MarioS271

use crate::lib::sync::irq_mutex::IrqMutex;
use crate::lib::sync::unchecked_cell::UncheckedCell;
use crate::mm::pmm::Pmm;
use crate::mm::vmm::address_space::AddressSpace;
use core::sync::atomic::{AtomicU64, Ordering};

// TODO: write-hot locks must not share a cache line with read-mostly fields

/// Holds memory management state such as slab allocators and virtual memory areas
#[repr(align(64))]
pub struct Mm {
    hhdm_offset: AtomicU64,
    pmm: UncheckedCell<IrqMutex<Pmm>>,
    kernel_addr_space: UncheckedCell<IrqMutex<AddressSpace>>
}

impl Mm {
    /// Constructor; initializes all values zeroed or uninited
    pub const fn new() -> Self {
        Self {
            hhdm_offset: AtomicU64::new(0),
            pmm: UncheckedCell::new(),
            kernel_addr_space: UncheckedCell::new()
        }
    }

    /// Sets the HHDM offset
    pub fn set_hhdm_offset(&self, offset: u64) {
        self.hhdm_offset.store(offset, Ordering::Release);
    }

    /// Move the given [`Pmm`] into `KSTATE::mm::pmm`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_pmm(&self, pmm: Pmm) {
        unsafe { self.pmm.init(IrqMutex::new(pmm)); }
    }

    /// Move the given [`AddressSpace`] into `KSTATE::mm::kernel_addr_space`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_kernel_addr_space(&self, kernel_addr_space: AddressSpace) {
        unsafe { self.kernel_addr_space.init(IrqMutex::new(kernel_addr_space)); }
    }

    /// Getter for the hhdm offset
    pub fn hhdm_offset(&self) -> u64 {
        let hhdm = self.hhdm_offset.load(Ordering::Acquire);

        #[cfg(feature = "debug-checks")]
        if hhdm == 0 {
            use crate::lib::panic::kernel_panic;
            use crate::lib::panic_codes::PanicCode;

            kernel_panic(
                PanicCode::UninitializedAccess,
                "Attempted to access KSTATE.mm.hhdm_offset before it was initialized"
            )
        }

        hhdm
    }

    /// Getter for `KSTATE::mm::pmm`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn pmm(&self) -> &IrqMutex<Pmm> {
        unsafe { self.pmm.get() }
    }

    /// Getter for `KSTATE::mm::kernel_address_space`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn kernel_addr_space(&self) -> &IrqMutex<AddressSpace> {
        unsafe { self.kernel_addr_space.get() }
    }
}
