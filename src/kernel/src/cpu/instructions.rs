// SPDX-License-Identifier: GPL-3.0-only
//! Common wrappers around arch-specific wrappers of CPU instructions
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")] pub use crate::arch::x86_64::cpu::instructions::*;

/// Calls [`halt_cpu`] in an infinite loop
pub fn halt_forever() -> ! {
    loop {
        halt_cpu();
    }
}
