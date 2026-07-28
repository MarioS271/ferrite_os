// SPDX-License-Identifier: GPL-3.0-only
//! Double-fault exception handler (vector 8).
//!
//! A double fault fires when a second exception occurs while the CPU is trying
//! to deliver the first one. The most common cause is a stack overflow that makes
//! the original exception handler's stack frame inaccessible. This handler runs on
//! IST stack 0 (set in the IDT) so it has its own guaranteed-valid stack even when
//! the main stack is corrupt.
//!
//! Authors: MarioS271

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the interrupt stack frame. The error code is always zero for double
/// faults and is ignored.
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
