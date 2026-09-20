// SPDX-License-Identifier: GPL-3.0-only
//! Memory subsystem: physical memory manager, virtual memory manager, and heap allocator.
//!
//! Authors: MarioS271


pub(crate) mod state;
pub(crate) mod mm_error;
pub(crate) mod vmm;
pub(crate) mod heap;
pub(crate) mod user;
pub(crate) mod kernel;

#[cfg(target_arch = "x86_64")] pub(crate) use crate::arch::x86_64::mm::layout;
#[cfg(target_arch = "x86_64")] pub(crate) use crate::arch::x86_64::mm::pmm;
