// SPDX-License-Identifier: GPL-3.0-only
//! Debug exception handler (vector 1).
//!
//! Authors: MarioS271

use x86_64::structures::idt::InterruptStackFrame;

/// No-op handler; returns immediately so the faulting instruction resumes.
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    return;
}
