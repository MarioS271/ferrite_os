//! arch/x86_64/interrupts/exceptions/_8_double_fault.rs
//! Double Fault Exception Handler
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame, _error_code: u64
) -> ! {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "{:#?}\n", isf);
    kernel_panic(
        PanicCode::DoubleFault,
        fmt_buffer.as_str(),
        true
    );
}
