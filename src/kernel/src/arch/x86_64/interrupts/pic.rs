//! arch/x86_64/interrupts/pic.rs
//! PIC (Programmable Interrupt Controller) Stuff
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use spin::Mutex;
use pic8259;

const PIC_MASTER_OFFSET: u8 = 0x20;
const PIC_SLAVE_OFFSET: u8 = 0x28;

static CHAINED_PICS: Mutex<pic8259::ChainedPics> = Mutex::new(unsafe { pic8259::ChainedPics::new(PIC_MASTER_OFFSET, PIC_SLAVE_OFFSET) });

pub fn init() {
    let mut _lock = CHAINED_PICS.lock();
    unsafe {
        _lock.initialize();
    }
}
