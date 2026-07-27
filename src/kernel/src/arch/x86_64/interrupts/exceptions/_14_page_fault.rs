//! arch/x86_64/interrupts/exceptions/_14_page_fault.rs
//! Page Fault Exception Handler
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::fmt::Write;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use crate::types::fmt_buffer::FmtBuffer;

pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    let cr2_value = Cr2::read_raw();
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "CR2: {}\nError Code:\n{:?}\n\n{:#?}", cr2_value, error_code, isf);
    kernel_panic(
        PanicCode::PageFault,
        fmt_buffer.as_str(),
        true
    );
}
