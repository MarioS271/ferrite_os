// SPDX-License-Identifier: GPL-3.0-only
//! IRQ0 handler — the 8253/8254 Programmable Interval Timer (PIT) tick.
//!
//! IRQ0 is wired to the PIT, which by default fires at roughly 18.2 Hz after BIOS
//! initialization. This is the earliest hardware interrupt the kernel handles and
//! will eventually drive the scheduler. The EOI must be sent before returning so
//! the PIC can deliver the next timer tick.
//!
//! Authors: MarioS271

use crate::arch::x86_64::interrupts::pic;
use x86_64::structures::idt::InterruptStackFrame;
use crate::kprint;

/// Acknowledge the timer tick and send EOI to the master PIC.
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    kprint!("a");
    pic::end_of_interrupt(pic::PIC_MASTER_OFFSET + 0);
}
