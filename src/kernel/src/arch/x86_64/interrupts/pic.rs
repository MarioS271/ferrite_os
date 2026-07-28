// SPDX-License-Identifier: GPL-3.0-only
//! 8259 Programmable Interrupt Controller (PIC) initialization and EOI signaling.
//!
//! The PC's legacy 8259 PIC delivers hardware IRQs to the CPU. By default the BIOS
//! maps IRQ0–7 to interrupt vectors 8–15, which collide with CPU exception vectors.
//! Initialization remaps the master PIC to vector 0x20 (32) and the slave to 0x28
//! (40), placing hardware IRQs safely above all 32 reserved exception vectors.
//!
//! After initialization all IRQ lines are masked except IRQ0 (the PIT timer).
//! Each IRQ handler must call [`end_of_interrupt`] before returning; without the
//! EOI signal, the PIC will not deliver any further IRQs on that or lower-priority
//! lines.
//!
//! Authors: MarioS271

use pic8259;
use crate::types::irq_mutex::IrqMutex;

/// I/O port for sending commands (including ISR read) to the master PIC.
pub const PIC_MASTER_CMD_PORT: u16 = 0x20;

/// Vector offset for IRQ0 on the master PIC after remapping. IRQ `n` (0–7) fires
/// at vector `PIC_MASTER_OFFSET + n`.
pub const PIC_MASTER_OFFSET: u8 = 0x20;

/// Vector offset for IRQ8 on the slave PIC after remapping. IRQ `n` (8–15) fires
/// at vector `PIC_SLAVE_OFFSET + (n - 8)`.
pub const PIC_SLAVE_OFFSET: u8 = 0x28;

/// The chained master+slave 8259 PIC pair, wrapped in an [`IrqMutex`] so IRQ
/// handlers can safely call `end_of_interrupt` without re-entering the lock.
static CHAINED_PICS: IrqMutex<pic8259::ChainedPics> = IrqMutex::new(unsafe { pic8259::ChainedPics::new(PIC_MASTER_OFFSET, PIC_SLAVE_OFFSET) });

/// Initialize and remap the PIC, then enable only IRQ0 (PIT timer).
///
/// The mask bytes passed to `write_masks` are bitmasks where bit `n` = 1 means
/// IRQ `n` is masked (disabled). `0xFE` on the master enables only IRQ0; `0xFF`
/// on the slave disables all slave lines.
pub fn init() {
    let mut _lock = CHAINED_PICS.lock();
    unsafe {
        _lock.initialize();
        _lock.write_masks(0xFE, 0xFF);
    }
}

/// Send an End-of-Interrupt (EOI) signal for the given interrupt vector.
///
/// `intr_vec` must be the actual vector number that fired (e.g.,
/// `PIC_MASTER_OFFSET + irq_line`). The PIC uses this to know which IRQ has
/// been serviced and can then deliver the next pending interrupt. Failing to call
/// this at the end of a hardware IRQ handler permanently blocks that IRQ line.
pub fn end_of_interrupt(intr_vec: u8) {
    let mut _lock = CHAINED_PICS.lock();

    unsafe {
        _lock.notify_end_of_interrupt(intr_vec);
    }
}
