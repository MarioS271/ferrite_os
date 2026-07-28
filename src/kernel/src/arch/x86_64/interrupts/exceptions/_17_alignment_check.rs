// SPDX-License-Identifier: GPL-3.0-only
//! Alignment-check exception handler (vector 17, `#AC`).
//!
//! Authors: MarioS271

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the error code (always 0) and interrupt stack frame.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame, error_code: u64
) {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "Error Code: {}\n{:#?}\n", error_code, isf);
    kernel_panic(
        PanicCode::AlignmentCheck,
        fmt_buffer.as_str(),
    );
}
