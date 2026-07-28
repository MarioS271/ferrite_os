// SPDX-License-Identifier: GPL-3.0-only
//! Page-fault exception handler (vector 14).
//!
//! A page fault fires when the CPU cannot translate a virtual address: the page
//! is not present, the access violates page protection flags, or a reserved bit
//! in a page table entry is set. The faulting virtual address is stored in CR2.
//!
//! Authors: MarioS271

use core::fmt::Write;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use crate::types::fmt_buffer::FmtBuffer;

/// Panic with the faulting virtual address (from CR2), the error code flags, and
/// the interrupt stack frame.
///
/// `error_code` is a bitfield (`PageFaultErrorCode`) that describes the nature of
/// the fault: whether the page was present, whether it was a write or instruction
/// fetch, and whether it came from user mode.
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
    );
}
