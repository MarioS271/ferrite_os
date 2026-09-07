// SPDX-License-Identifier: GPL-3.0-only
//! Re-Exports of arch-specific userspace jump stuff
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")] pub use crate::arch::x86_64::cpu::userspace::*;
