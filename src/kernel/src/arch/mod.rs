//! arch/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains architecture-specific things such as GDT init for x86_64

#[cfg(target_arch = "x86_64")] mod x86_64;
#[cfg(target_arch = "x86_64")] pub(crate) use x86_64::*;

#[cfg(target_arch = "aarch64")] mod aarch64;
#[cfg(target_arch = "aarch64")] pub(crate) use aarch64::*;