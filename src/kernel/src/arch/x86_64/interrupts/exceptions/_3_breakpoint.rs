// SPDX-License-Identifier: GPL-3.0-only
//! Breakpoint exception handler (vector 3, `int3`).
//!
//! Authors: MarioS271

use x86_64::structures::idt::InterruptStackFrame;

/// No-op handler; returns immediately so execution resumes after the `int3`.
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    return;
}
