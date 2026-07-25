//! arch/x86_64/instructions.rs
//! x86_64 asm instruction wrappers
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use x86_64::instructions;

// unsafe/asm justification for all:
// no memory is manipulated, only cpu state is changed

pub fn enable_interrupts() {
    instructions::interrupts::enable();
}

pub fn disable_interrupts() {
    instructions::interrupts::disable();
}

pub fn halt_cpu() {
    instructions::hlt();
}