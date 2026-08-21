// SPDX-License-Identifier: GPL-3.0-only
//! Init code
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")] mod x86_64;
#[cfg(target_arch = "x86_64")] pub(crate) use x86_64::*;
