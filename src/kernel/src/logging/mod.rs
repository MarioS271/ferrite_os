// SPDX-License-Identifier: GPL-3.0-only
//! Kernel logging: serial output and the [`kprint!`] macro.
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")] mod x86_64;
#[cfg(target_arch = "x86_64")] pub(crate) use x86_64::*;

pub(crate) mod kprint;
pub(crate) mod _serial;
