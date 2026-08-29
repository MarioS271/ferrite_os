// SPDX-License-Identifier: GPL-3.0-only
//! Memory management subcategory placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::mem::pmm::Pmm;
use crate::mem::vmm::address_space::AddressSpace;
use crate::panic::kernel_panic;
use crate::types::irq_mutex::IrqMutex;
use crate::types::panic_codes::PanicCode;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU64, Ordering};

/// Holds memory management state such as slab allocators and virtual memory areas.
pub struct Mm {
    hhdm_offset: AtomicU64,
    pmm: MaybeUninit<IrqMutex<Pmm>>,
    kernel_addr_space: MaybeUninit<IrqMutex<AddressSpace>>
}

impl Mm {
    /// Constructor; initializes all values zeroed or empty
    pub const fn new() -> Self {
        Self {
            hhdm_offset: AtomicU64::new(0),
            pmm: MaybeUninit::uninit(),
            kernel_addr_space: MaybeUninit::uninit()
        }
    }

    /// Sets the hhdm offset
    pub fn set_hhdm_offset(&self, offset: u64) {
        self.hhdm_offset.store(offset, Ordering::Release);
    }

    /// Sets the PMM
    pub fn init_pmm(&mut self, pmm: Pmm) {
        self.pmm.write(IrqMutex::new(pmm));
    }

    /// Sets the kernel address space
    pub fn init_kernel_addr_space(&mut self, kernel_addr_space: AddressSpace) {
        self.kernel_addr_space.write(IrqMutex::new(kernel_addr_space));
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

    /// Getter for KSTATE.mm.pmm
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    pub fn pmm(&self) -> &IrqMutex<Pmm> {
        unsafe { self.pmm.assume_init_ref() }
    }

    /// Getter for KSTATE.mm.kernel_address_space
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    pub fn kernel_addr_space(&self) -> &IrqMutex<AddressSpace> {
        unsafe { self.kernel_addr_space.assume_init_ref() }
    }
}
