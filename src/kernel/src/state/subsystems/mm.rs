// SPDX-License-Identifier: GPL-3.0-only
//! Memory management subcategory placeholder for [`KState`].
//!
//! Authors: MarioS271

use core::sync::atomic::{AtomicU64, Ordering};
use spin::once::Once;
use crate::mem::pmm::Pmm;
use crate::mem::vmm::Vmm;
use crate::types::irq_mutex::IrqMutex;

/// Holds memory management state such as slab allocators and virtual memory areas.
pub struct Mm {
    hhdm_offset: AtomicU64,
    pub pmm: Once<IrqMutex<Pmm>>,
    pub vmm: Once<IrqMutex<Vmm>>
}

impl Mm {
    /// Construct with an empty pmm and vmm
    pub const fn new() -> Self {
        Self {
            hhdm_offset: AtomicU64::new(0),
            pmm: Once::new(),
            vmm: Once::new()
        }
    }

    /// Setter for the hhdm offset
    pub fn set_hhdm_offset(&self, offset: u64) {
        self.hhdm_offset.store(offset, Ordering::Release)
    }

    /// Getter for the hhdm offset
    pub fn hhdm_offset(&self) -> u64 {
        self.hhdm_offset.load(Ordering::Acquire)
    }
}
