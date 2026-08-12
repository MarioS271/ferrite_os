// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 memory backend: four-level page table VMM and helper utilities.
//!
//! Authors: MarioS271

mod vmm;
mod paging;

pub use vmm::*;
pub use paging::*;
