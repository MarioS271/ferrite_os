// SPDX-License-Identifier: GPL-3.0-only
//! Invalid-opcode exception handler (vector 6, `#UD`).
//!
//! Fires when the CPU encounters an instruction it does not recognize, or one that
//! is not valid in the current mode (e.g., a legacy instruction in 64-bit mode).
//! Can also be used intentionally as a software trap via the `ud2` instruction.
//!
//! Authors: MarioS271

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the interrupt stack frame.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame
) {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "{:#?}\n", isf);
    kernel_panic(
        PanicCode::InvalidOpcode,
        fmt_buffer.as_str(),
    );
}
