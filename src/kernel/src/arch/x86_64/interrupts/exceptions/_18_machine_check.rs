//! arch/x86_64/interrupts/exceptions/_18_machine_check.rs
//! Machine Check Exception Handler
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only
//!
//! TODO: replace kernel_panic with direct serial write bypassing all locks
//! (MCE can fire while any lock is held; kernel_panic may deadlock — see TODO #21)

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::lib::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame
) -> ! {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "{:#?}\n", isf);
    kernel_panic(
        PanicCode::MachineCheck,
        fmt_buffer.as_str(),
        true
    );
}
