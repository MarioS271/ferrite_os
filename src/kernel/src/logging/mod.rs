//! logging/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains code for logging (serial logging, kprint)

#[cfg(target_arch = "x86_64")] mod x86_64;
#[cfg(target_arch = "x86_64")] pub(crate) use x86_64::*;

pub(crate) mod kprint;
pub(crate) mod _serial;
