//! Breakpoint exception handler (vector 3, `int3`).
//!
//! Triggered by the `int3` instruction, typically inserted by debuggers as a
//! software breakpoint. Currently a no-op — no debugger is attached, so execution
//! resumes immediately after the `int3` instruction.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use x86_64::structures::idt::InterruptStackFrame;

/// No-op handler; returns immediately so execution resumes after the `int3`.
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    return;
}
