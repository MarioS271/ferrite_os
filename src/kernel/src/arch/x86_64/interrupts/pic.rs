// SPDX-License-Identifier: GPL-3.0-only
//! 8259 Programmable Interrupt Controller (PIC): remaps hardware IRQ vectors and
//! signals end-of-interrupt.
//!
//! Authors: MarioS271

use pic8259;
use crate::types::irq_mutex::IrqMutex;

/// I/O port for sending commands (including ISR read) to the master PIC.
pub const PIC_MASTER_CMD_PORT: u16 = 0x20;

/// Vector offset for master-PIC IRQs after remapping; IRQ `n` fires at `PIC_MASTER_OFFSET + n`.
pub const PIC_MASTER_OFFSET: u8 = 0x20;

/// Vector offset for slave-PIC IRQs after remapping; IRQ `n` fires at `PIC_SLAVE_OFFSET + (n - 8)`.
pub const PIC_SLAVE_OFFSET: u8 = 0x28;

/// The chained master+slave PIC pair, lockable from IRQ handlers.
static CHAINED_PICS: IrqMutex<pic8259::ChainedPics> = IrqMutex::new(unsafe { pic8259::ChainedPics::new(PIC_MASTER_OFFSET, PIC_SLAVE_OFFSET) });

/// Initialize and remap the PIC, then enable only IRQ0 (PIT timer).
pub fn init() {
    let mut _lock = CHAINED_PICS.lock();
    unsafe {
        _lock.initialize();
        _lock.write_masks(0xFE, 0xFF);
    }
}

/// Send an End-of-Interrupt (EOI) for the vector that fired; every hardware IRQ
/// handler must call this before returning or that IRQ line stays blocked.
pub fn end_of_interrupt(intr_vec: u8) {
    let mut _lock = CHAINED_PICS.lock();

    unsafe {
        _lock.notify_end_of_interrupt(intr_vec);
    }
}
