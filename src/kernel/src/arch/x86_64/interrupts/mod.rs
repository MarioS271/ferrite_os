// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 interrupt infrastructure: exception handlers, IRQ handlers, and PIC initialization.
//!
//! Authors: MarioS271

pub(crate) mod exceptions;
pub(crate) mod irqs;
pub(crate) mod pic;
