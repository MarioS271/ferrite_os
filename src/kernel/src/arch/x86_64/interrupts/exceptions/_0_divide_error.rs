// SPDX-License-Identifier: GPL-3.0-only
//! Divide-error exception handler (vector 0).
//!
//! Fires when the CPU executes a `div` or `idiv` instruction with a zero divisor,
//! or when the quotient is too large for the destination register.
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
        PanicCode::DivideError,
        fmt_buffer.as_str(),
    );
}
