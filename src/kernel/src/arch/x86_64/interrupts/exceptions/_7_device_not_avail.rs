//! Device-not-available exception handler (vector 7, `#NM`).
//!
//! Fires when an FPU or SSE instruction is executed while the CR0.TS (Task Switched)
//! flag is set. Typically used by OS schedulers to implement lazy FPU context
//! switching. The kernel does not currently use lazy FPU switching, so this fires
//! only on unexpected conditions.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

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
        PanicCode::DeviceNotAvail,
        fmt_buffer.as_str(),
    );
}
