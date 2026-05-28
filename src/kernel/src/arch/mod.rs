//! arch/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains architecture-specific things such as GDT init for x86_64

#[cfg(target_arch = "x86_64")] mod x86_64;
#[cfg(target_arch = "aarch64")] mod aarch64;

pub(crate) fn init() {
    #[cfg(target_arch = "x86_64")] x86_64::init();
    #[cfg(target_arch = "aarch64")] aarch64::init();

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    compile_error!("Attempting to compile to unsupported target architecture");
}
