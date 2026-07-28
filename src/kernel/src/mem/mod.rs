// SPDX-License-Identifier: GPL-3.0-only
//! Memory subsystem: physical memory manager, virtual memory manager, and heap allocator.
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")] mod x86_64;
#[cfg(target_arch = "x86_64")] pub(crate) use x86_64::*;

pub(crate) mod pmm;
pub(crate) mod heap;
