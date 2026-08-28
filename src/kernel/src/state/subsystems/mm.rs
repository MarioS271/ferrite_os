// SPDX-License-Identifier: GPL-3.0-only
//! Memory management subcategory placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::mem::pmm::Pmm;
use crate::mem::vmm::address_space::AddressSpace;
use crate::panic::kernel_panic;
use crate::types::irq_mutex::IrqMutex;
use crate::types::panic_codes::PanicCode;
use spin::once::Once;

/// Holds memory management state such as slab allocators and virtual memory areas.
pub struct Mm {
    hhdm_offset: Once<u64>,
    pmm: Once<IrqMutex<Pmm>>,
    kernel_addr_space: Once<IrqMutex<AddressSpace>>
}

impl Mm {
    /// Constructor; initializes all values zeroed or empty
    pub const fn new() -> Self {
        Self {
            hhdm_offset: Once::new(),
            pmm: Once::new(),
            kernel_addr_space: Once::new()
        }
    }

    /// Sets the hhdm offset
    pub fn set_hhdm_offset(&self, offset: u64) {
        self.hhdm_offset.call_once(|| offset);
    }

    /// Sets the PMM
    pub fn init_pmm(&self, pmm: Pmm) {
        self.pmm.call_once(|| IrqMutex::new(pmm));
    }

    /// Sets the kernel address space
    pub fn init_kernel_addr_space(&self, kernel_addr_space: AddressSpace) {
        self.kernel_addr_space.call_once(|| IrqMutex::new(kernel_addr_space));
    }

    /// Getter for the hhdm offset
    pub fn hhdm_offset(&self) -> u64 {
        *self.hhdm_offset.get().unwrap_or_else(|| kernel_panic(
            PanicCode::InitFailure,
            "Attempted to access KSTATE.mm.hhdm_offset before it was initialized"
        ))
    }

    /// Getter for the PMM's IrqMutex
    pub fn pmm(&self) -> &IrqMutex<Pmm> {
        self.pmm.get().unwrap_or_else(|| kernel_panic(
            PanicCode::InitFailure,
            "Attempted to access KSTATE.mm.pmm before it was initialized"
        ))
    }

    pub fn kernel_addr_space(&self) -> &IrqMutex<AddressSpace> {
        self.kernel_addr_space.get().unwrap_or_else(|| kernel_panic(
            PanicCode::InitFailure,
            "Attempted to access KSTATE.mm.kernel_addr_space before it was initialized"
        ))
    }
}
