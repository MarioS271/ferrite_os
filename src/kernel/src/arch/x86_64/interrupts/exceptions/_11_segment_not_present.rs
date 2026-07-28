// SPDX-License-Identifier: GPL-3.0-only
//! Segment-not-present exception handler (vector 11, `#NP`).
//!
//! Fires when the CPU tries to load a segment register from a descriptor whose
//! Present bit is 0. The error code is a segment selector index indicating the
//! offending descriptor.
//!
//! Authors: MarioS271

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the segment selector error code and interrupt stack frame.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame, error_code: u64
) {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "Error Code: {}\n{:#?}\n", error_code, isf);
    kernel_panic(
        PanicCode::SegmentNotPresent,
        fmt_buffer.as_str(),
    );
}
