// SPDX-License-Identifier: GPL-3.0-only
//! Double-fault exception handler (vector 8).
//!
//! Authors: MarioS271

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the interrupt stack frame.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame, _error_code: u64
) -> ! {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "{:#?}\n", isf);
    kernel_panic(
        PanicCode::DoubleFault,
        fmt_buffer.as_str(),
    );
}
