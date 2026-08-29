// SPDX-License-Identifier: GPL-3.0-only
//! Kernel logging: serial output and the [`kprint!`] macro.
//!
//! Authors: MarioS271

pub(super) mod x86_64;

pub(crate) mod kprint;
pub(crate) mod serial;
