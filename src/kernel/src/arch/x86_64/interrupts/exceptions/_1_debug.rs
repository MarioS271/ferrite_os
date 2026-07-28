//! Debug exception handler (vector 1).
//!
//! Fires when a hardware breakpoint or single-step condition is triggered. Currently
//! a no-op: the kernel has no debugger attached, so the exception is silently ignored
//! and execution resumes. This handler runs on IST stack 1 so a corrupted stack at
//! the breakpoint site does not prevent the handler from running.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use x86_64::structures::idt::InterruptStackFrame;

/// No-op handler; returns immediately so the faulting instruction resumes.
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    return;
}
