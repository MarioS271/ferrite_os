// SPDX-License-Identifier: GPL-3.0-only
//! Memory management subcategory placeholder for [`KState`].
//!
//! Authors: MarioS271

use spin::once::Once;
use crate::mem::pmm::Pmm;
use crate::mem::vmm::Vmm;
use crate::types::irq_mutex::IrqMutex;

/// Holds memory management state such as slab allocators and virtual memory areas.
pub struct Mm {
    pub pmm: Once<IrqMutex<Pmm>>,
    pub vmm: Once<IrqMutex<Vmm>>
}
impl Mm {
    /// Construct with an empty pmm and vmm
    pub const fn new() -> Self {
        Self {
            pmm: Once::new(),
            vmm: Once::new()
        }
    }
}