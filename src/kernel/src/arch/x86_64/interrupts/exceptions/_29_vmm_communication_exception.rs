// SPDX-License-Identifier: GPL-3.0-only
//! VMM communication exception handler (vector 29, `#VC`, AMD SVM).
//!
//! AMD-specific: fires when a SEV-ES (Secure Encrypted Virtualization - Encrypted
//! State) guest needs to communicate with the hypervisor. Not relevant for a
//! non-SEV kernel, so treated as a fatal exception.
//!
//! Authors: MarioS271

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
        PanicCode::VmmCommunicationException,
        fmt_buffer.as_str(),
    );
}
