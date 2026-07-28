//! Stack-segment fault exception handler (vector 12, `#SS`).
//!
//! Fires on stack-related violations: loading SS with a non-present descriptor,
//! or a stack access that exceeds the stack segment's limit. The error code is
//! the SS segment selector index, or 0 for a limit violation.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the error code and interrupt stack frame.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame, error_code: u64
) {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "Error Code: {}\n{:#?}\n", error_code, isf);
    kernel_panic(
        PanicCode::StackSegmentFault,
        fmt_buffer.as_str(),
    );
}
