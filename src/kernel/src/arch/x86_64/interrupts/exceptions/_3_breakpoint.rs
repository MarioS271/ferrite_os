//! arch/x86_64/interrupts/exceptions/_3_breakpoint.rs
//! Breakpoint Interrupt
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    return;
}
