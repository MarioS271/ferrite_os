//! Security exception handler (vector 30, `#SX`, AMD SVM).
//!
//! AMD-specific: fired by the hypervisor to inject a security-related event into
//! a guest (e.g., an INIT or SIPI signal). Not applicable to a bare-metal kernel
//! that is not running as an AMD SVM guest.
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
        PanicCode::SecurityException,
        fmt_buffer.as_str(),
    );
}
