//! x87 floating-point exception handler (vector 16, `#MF`).
//!
//! Fires when an unmasked x87 FPU exception condition (divide-by-zero, overflow,
//! underflow, precision, etc.) is detected by a subsequent floating-point or `wait`
//! instruction. The FPU status word must be read to determine the specific cause.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

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
        PanicCode::X87FloatingPoint,
        fmt_buffer.as_str(),
    );
}
