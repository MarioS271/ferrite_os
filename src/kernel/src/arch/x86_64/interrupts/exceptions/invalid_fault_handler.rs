//! arch/x86_64/interrupts/exceptions/invalid_fault_handler.rs
//! Handler for interrupts such as #5 (bound check, doesn't exist in long mode) or #9 (reserved, currently invalid exception)
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

pub extern "x86-interrupt" fn handler<const vec: usize>(
    isf: InterruptStackFrame
) {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(
        &mut fmt_buffer,
        "WARNING: Vector #{vec} fired which should be\nimpossible (reserved exception?), this suggests possible IDT corruption or similar\n\n{:#?}\n",
        isf
    );
    kernel_panic(
        PanicCode::IllegalInterrupt,
        fmt_buffer.as_str(),
        true
    );
}
