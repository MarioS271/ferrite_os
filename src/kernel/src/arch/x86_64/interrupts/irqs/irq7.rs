// SPDX-License-Identifier: GPL-3.0-only
//! Spurious PIC interrupt handler
//!
//! Authors: MarioS271

use crate::arch::x86_64::interrupts::pic;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

/// Check whether IRQ7 is spurious; send EOI only if the ISR bit 7 is set (real IRQ)
pub extern "x86-interrupt" fn handler(
    _: InterruptStackFrame
) {
    let mut port: Port<u8> = Port::new(pic::PIC_MASTER_CMD_PORT);
    let isr;

    // Safety:
    // - The port at PIC_MASTER_CMD_PORT is the correct PIC command port per x86_64 standards
    // - 0x0B is the OCW3 command to be able to read the ISR register
    unsafe {
        port.write(0x0B);
         isr = port.read();
    }

    if isr & (1 << 7) == 0 {
        return;
    }

    pic::end_of_interrupt(pic::PIC_MASTER_OFFSET + 7);
}
