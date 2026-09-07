// SPDX-License-Identifier: GPL-3.0-only
//! Safe wrappers around x86_64 privileged instructions
//!
//! Authors: MarioS271

use core::arch::asm;

/// Set the CPU's Interrupt Flag via `sti`, unmasking hardware IRQs
pub fn enable_interrupts() {
    unsafe { asm!("sti", options(nostack, nomem)) };
}

/// Clear the CPU's Interrupt Flag via `cli`, masking hardware IRQs (CPU exceptions like `#PF`, `#MC` or NMI can still fire)
pub fn disable_interrupts() {
    unsafe { asm!("cli", options(nostack, nomem)) };
}

/// Halt the CPU until an interrupt arrives
pub fn halt_cpu() {
    unsafe { asm!("hlt", options(nostack)) }
}
