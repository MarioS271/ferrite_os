// SPDX-License-Identifier: GPL-3.0-only
//! Arch-Independent VMM Code (traits, AddressSpace, VMAs)
//!
//! Authors: MarioS271

mod vmm;
pub(crate) use vmm::*;

pub(crate) mod traits;
pub(crate) mod address_space;
pub(crate) mod vma;
pub(crate) mod helpers;

#[cfg(target_arch = "x86_64")] use super::x86_64::vmm::*;
