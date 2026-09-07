// SPDX-License-Identifier: GPL-3.0-only
//! Kernel Print: the [`macros!`] macro, kernel serial logging, kernel log ringbuffer and more
//!
//! Authors: MarioS271

pub(crate) mod kernel_writer;
pub(crate) mod log_entry;
pub(crate) mod macros;
pub(crate) mod ringbuffer;
pub(crate) mod state;
mod receivers;
