// SPDX-License-Identifier: GPL-3.0-only
//! Safe wrappers around x86_64 privileged instructions.
//!
//! Authors: MarioS271

use x86_64::instructions;

/// Set the CPU's Interrupt Flag via `sti`; only call once the IDT and PIC are initialized.
pub fn enable_interrupts() {
    instructions::interrupts::enable();
}

/// Clear the CPU's Interrupt Flag via `cli`, masking hardware IRQs (NMIs and exceptions still fire).
pub fn disable_interrupts() {
    instructions::interrupts::disable();
}

/// Issue `hlt`, suspending the CPU until the next interrupt.
pub fn halt_cpu() {
    instructions::hlt();
}