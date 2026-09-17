// SPDX-License-Identifier: GPL-3.0-only
//! Architecture-specific code
//! Each arch's top level `mod.rs` should provide a public `boot` function to make initializing
//! that arch easier
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")] pub(crate) mod x86_64;

#[cfg(target_arch = "aarch64")] pub(crate) mod aarch64;

#[inline(always)]
pub(crate) fn init(boot_info: &crate::lib::types::boot_info::BootInfo) {
    #[cfg(target_arch = "x86_64")] x86_64::boot::init(boot_info);
    #[cfg(target_arch = "aarch64")] aarch64::boot::init(boot_info);
}
