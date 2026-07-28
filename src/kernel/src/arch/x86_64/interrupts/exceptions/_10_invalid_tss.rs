// SPDX-License-Identifier: GPL-3.0-only
//! Invalid-TSS exception handler (vector 10, `#TS`).
//!
//! Fires when the CPU encounters a problem loading a segment selector from the TSS
//! — e.g., a bad limit or an invalid IST pointer. The error code is a segment
//! selector index identifying which selector caused the fault.
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
        PanicCode::InvalidTss,
        fmt_buffer.as_str(),
    );
}
