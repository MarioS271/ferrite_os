// SPDX-License-Identifier: GPL-3.0-only
//! Memory subsystem: physical memory manager, virtual memory manager, and heap allocator.
//!
//! Authors: MarioS271

pub(super) mod x86_64;

pub(crate) mod vmm;
pub(crate) mod heap;

pub(crate) use x86_64::pmm;
