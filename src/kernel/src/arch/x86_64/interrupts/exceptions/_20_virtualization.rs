//! Virtualization exception handler (vector 20, `#VE`).
//!
//! Fires on Intel VMX EPT violations or on AMD virtualization events when the
//! processor is operating as a guest and the hypervisor has configured these
//! events to be injected. The kernel does not currently run as a VMX guest, so
//! this exception should not occur in practice.
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
        PanicCode::Virtualization,
        fmt_buffer.as_str(),
    );
}
