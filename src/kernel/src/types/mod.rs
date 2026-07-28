// SPDX-License-Identifier: GPL-3.0-only
//! Shared kernel types: `IrqMutex`, `FmtBuffer`, `AlignedStack`, and panic codes.
//!
//! Authors: MarioS271

pub(crate) mod panic_codes;
pub(crate) mod aligned_stack;
pub(crate) mod irq_mutex;
pub(crate) mod fmt_buffer;
