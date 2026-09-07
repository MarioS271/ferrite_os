// SPDX-License-Identifier: GPL-3.0-only
//! Double-fault exception handler (vector 8, `#DF`).
//!
//! Authors: MarioS271

use crate::lib::panic_codes::PanicCode;
use crate::lib::types::fmt_buffer::FmtBuffer;
use crate::lib::panic::kernel_panic;
use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;

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
