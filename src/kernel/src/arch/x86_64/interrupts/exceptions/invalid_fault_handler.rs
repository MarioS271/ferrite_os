// SPDX-License-Identifier: GPL-3.0-only
//! Generic handler for interrupt vectors that cannot legally fire in 64-bit long mode.
//!
//! Authors: MarioS271

use crate::lib::panic_codes::PanicCode;
use crate::lib::types::fmt_buffer::FmtBuffer;
use crate::lib::panic::kernel_panic;
use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;

/// Panic identifying the impossible vector (`VEC`) that fired, with the interrupt stack frame.
pub extern "x86-interrupt" fn handler<const VEC: usize>(
    isf: InterruptStackFrame
) {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(
        &mut fmt_buffer,
        "WARNING: Vector #{VEC} fired which should be\nimpossible (reserved exception?), this suggests possible IDT corruption or similar\n\n{:#?}\n",
        isf
    );
    kernel_panic(
        PanicCode::IllegalInterrupt,
        fmt_buffer.as_str(),
    );
}
