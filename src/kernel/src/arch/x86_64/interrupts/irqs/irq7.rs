//! IRQ7 handler — spurious interrupt detection.
//!
//! The 8259 PIC can generate a spurious IRQ7 when an IRQ is cancelled (e.g., the
//! signal disappears before the PIC fully acknowledges it). Unlike a real IRQ7, a
//! spurious interrupt must NOT receive an EOI — sending one would incorrectly tell
//! the PIC that a real interrupt was handled. The handler detects spurious interrupts
//! by reading the In-Service Register (ISR): if bit 7 is clear, the interrupt was
//! spurious and the handler returns without sending EOI.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::arch::x86_64::interrupts::pic;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

/// Check whether IRQ7 is spurious; send EOI only if the ISR bit 7 is set (real IRQ).
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    // Safe because raw asm is encapsulated in the trusted x86_64 crate
    unsafe {
        let mut port: Port<u8> = Port::new(pic::PIC_MASTER_CMD_PORT);
        port.write(0x0B);
        let isr: u8 = port.read();
        if isr & (1 << 7) == 0 {
            return;
        }
    }

    pic::end_of_interrupt(pic::PIC_MASTER_OFFSET + 7);
}
