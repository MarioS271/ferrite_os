// SPDX-License-Identifier: GPL-3.0-only
//! Architecture-specific subsystems: GDT, TSS, IDT, and interrupt controllers.
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")] mod x86_64;
#[cfg(target_arch = "x86_64")] pub(crate) use x86_64::*;

#[cfg(target_arch = "aarch64")] mod aarch64;
#[cfg(target_arch = "aarch64")] pub(crate) use aarch64::*;
