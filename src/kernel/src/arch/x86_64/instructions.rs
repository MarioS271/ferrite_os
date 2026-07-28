// SPDX-License-Identifier: GPL-3.0-only
//! Safe wrappers around x86_64 privileged instructions.
//!
//! Authors: MarioS271

use x86_64::instructions;

/// Set the CPU's Interrupt Flag (IF) via the `sti` instruction.
///
/// After this returns, the CPU will begin accepting hardware IRQs. Must only be
/// called once the IDT and PIC are fully initialized; calling it earlier risks
/// delivering an interrupt with no registered handler.
pub fn enable_interrupts() {
    instructions::interrupts::enable();
}

/// Clear the CPU's Interrupt Flag (IF) via the `cli` instruction.
///
/// After this returns, hardware IRQs are masked until `enable_interrupts` is
/// called again. NMIs and exceptions still fire regardless of IF.
pub fn disable_interrupts() {
    instructions::interrupts::disable();
}

/// Issue a `hlt` instruction, suspending the CPU until the next interrupt.
///
/// Used in idle loops to avoid burning CPU cycles. Execution resumes at the
/// instruction after `hlt` when an interrupt (or NMI) is delivered.
pub fn halt_cpu() {
    instructions::hlt();
}