//! arch/x86_64/interrupts/irqs/irq7.rs
//! Handler for IRQ7 (For now only handles spurious IRQs)
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::arch::x86_64::interrupts::pic;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

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
