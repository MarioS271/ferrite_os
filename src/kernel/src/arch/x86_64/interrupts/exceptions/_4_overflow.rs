// SPDX-License-Identifier: GPL-3.0-only
//! Overflow trap handler (vector 4, `into` instruction).
//!
//! Fires when the `into` instruction is executed and the overflow flag (OF) is set.
//! Rare in modern 64-bit code since `into` is not encodable in 64-bit mode; this
//! handler exists for completeness.
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
        PanicCode::Overflow,
        fmt_buffer.as_str(),
    );
}
