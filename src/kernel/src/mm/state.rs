// SPDX-License-Identifier: GPL-3.0-only
//! Memory management subcategory for [`KState`]
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::lib::sync::irq_mutex::IrqMutex;
use crate::lib::sync::unchecked_cell::UncheckedCell;
use crate::mm::pmm::Pmm;
use crate::mm::vmm::address_space::AddressSpace;
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "debug-checks")] use crate::lib::panic::kernel_panic;
#[cfg(feature = "debug-checks")] use crate::lib::panic_codes::PanicCode;

// TODO: write-hot locks must not share a cache line with read-mostly fields

/// Holds memory management state such as slab allocators and virtual memory areas
#[repr(align(64))]
pub struct Mm {
    hhdm_offset: AtomicU64,
    pmm: UncheckedCell<IrqMutex<Pmm>>,
    kernel_addr_space: UncheckedCell<IrqMutex<AddressSpace>>,
    next_kernel_stack: AtomicU64
}

impl Mm {
    /// Constructor; initializes all values zeroed or uninited
    pub const fn new() -> Self {
        Self {
            hhdm_offset: AtomicU64::new(0),
            pmm: UncheckedCell::new(),
            kernel_addr_space: UncheckedCell::new(),
            next_kernel_stack: AtomicU64::new(0)
        }
    }

    /// Initialize `Mm`'s default values
    pub fn init(&self) {
        self.next_kernel_stack.store(super::layout::KERNEL_STACKS_BASE.as_u64(), Ordering::Release);
    }

    /// Sets the HHDM offset
    pub fn set_hhdm_offset(&self, offset: u64) {
        self.hhdm_offset.store(offset, Ordering::Release);
    }

    /// Move the given [`Pmm`] into `Mm::pmm`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_pmm(&self, pmm: Pmm) {
        unsafe { self.pmm.init(IrqMutex::new(pmm)); }
    }

    /// Move the given [`AddressSpace`] into `Mm::kernel_addr_space`
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
            kernel_panic(
                PanicCode::UninitializedAccess,
                "Attempted to access KSTATE.mm.hhdm_offset before it was initialized"
            )
        }

        hhdm
    }

    /// Getter for `Mm::pmm`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn pmm(&self) -> &IrqMutex<Pmm> {
        unsafe { self.pmm.get() }
    }

    /// Getter for `Mm::kernel_address_space`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn kernel_addr_space(&self) -> &IrqMutex<AddressSpace> {
        unsafe { self.kernel_addr_space.get() }
    }
    
    /// Get the starting address of the next kernel stack and increment it
    pub fn alloc_kernel_stack_slot(&self) -> VirtAddr {
        use super::layout::*;

        let addr = self.next_kernel_stack.fetch_add(KERNEL_STACK_SLOT_SIZE, Ordering::Relaxed);
        let addr_virt = VirtAddr::new(addr);

        #[cfg(feature = "debug-checks")]
        {
            if addr_virt < KERNEL_STACKS_BASE {
                kernel_panic(
                    PanicCode::UninitializedAccess,
                    "Attempted to get the next kernel stack address before Mm::init was called"
                );
            }

            if addr_virt > KERNEL_STACKS_BASE + KERNEL_STACKS_SIZE - KERNEL_STACK_SLOT_SIZE {
                kernel_panic(
                    PanicCode::OutOfVirtualMemory,
                    "Could not allocate a kernel stack, out of virtual memory for kernel stacks (somehow)"
                );
            }
        }

        addr_virt
    }
}
