//! arch/x86_64/interrupts/irqs/irq0_timer.rs
//! Handler for IRQ0 (System Timer IRQ)
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::arch::x86_64::interrupts::pic;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    pic::end_of_interrupt(pic::PIC_MASTER_OFFSET + 0);
}
