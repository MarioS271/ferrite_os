//! arch/x86_64/interrupts/pic.rs
//! PIC (Programmable Interrupt Controller) Stuff
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use pic8259;
use crate::types::irq_mutex::IrqMutex;

pub const PIC_MASTER_CMD_PORT: u16 = 0x20;
pub const PIC_MASTER_OFFSET: u8 = 0x20;
pub const PIC_SLAVE_OFFSET: u8 = 0x28;

static CHAINED_PICS: IrqMutex<pic8259::ChainedPics> = IrqMutex::new(unsafe { pic8259::ChainedPics::new(PIC_MASTER_OFFSET, PIC_SLAVE_OFFSET) });

pub fn init() {
    let mut _lock = CHAINED_PICS.lock();
    unsafe {
        _lock.initialize();
        _lock.write_masks(0xFE, 0xFF);
    }
}

pub fn end_of_interrupt(intr_vec: u8) {
    let mut _lock = CHAINED_PICS.lock();

    unsafe {
        _lock.notify_end_of_interrupt(intr_vec);
    }
}
