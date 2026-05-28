//! arch/x86_64/instructions.rs
//! x86_64 asm instruction wrappers
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::arch::asm;

// unsafe/asm justification for all:
// no memory is manipulated, only cpu state is changed

pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack, nomem));
    }
}

pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack, nomem));
    }
}

pub fn halt_cpu() {
    unsafe {
        asm!("hlt", options(nostack, nomem));
    }
}