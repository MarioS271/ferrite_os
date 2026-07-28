// SPDX-License-Identifier: GPL-3.0-only
//! SIMD floating-point exception handler (vector 19, `#XM`/`#XF`).
//!
//! Fires when an unmasked SSE/SSE2/SSE3 floating-point exception occurs. The
//! specific cause can be read from the MXCSR register's exception flags, but the
//! kernel currently treats all cases as fatal.
//!
//! Authors: MarioS271

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
        PanicCode::SimdFloatingPoint,
        fmt_buffer.as_str(),
    );
}
