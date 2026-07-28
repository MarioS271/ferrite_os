// SPDX-License-Identifier: GPL-3.0-only
//! General Protection Fault exception handler (vector 13).
//!
//! A GPF fires for a broad class of protection violations: accessing a segment
//! with insufficient privilege, executing a privileged instruction from user mode,
//! writing to a read-only page (before NX/WP enforcement), or passing a bad
//! segment selector. The error code encodes the segment selector index when the
//! fault is selector-related, or zero otherwise.
//!
//! Authors: MarioS271

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the error code (selector index or 0) and the interrupt stack frame.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame, error_code: u64
) {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "Error Code: {}\n{:#?}\n", error_code, isf);
    kernel_panic(
        PanicCode::GeneralProtectionFault,
        fmt_buffer.as_str(),
    );
}
