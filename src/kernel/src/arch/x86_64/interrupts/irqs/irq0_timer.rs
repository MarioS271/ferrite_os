// SPDX-License-Identifier: GPL-3.0-only
//! IRQ0 handler — the 8253/8254 Programmable Interval Timer (PIT) tick.
//!
//! Authors: MarioS271

use crate::arch::x86_64::interrupts::pic;
use x86_64::structures::idt::InterruptStackFrame;

/// Acknowledge the timer tick and send EOI to the master PIC.
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    pic::end_of_interrupt(pic::PIC_MASTER_OFFSET + 0);
}
