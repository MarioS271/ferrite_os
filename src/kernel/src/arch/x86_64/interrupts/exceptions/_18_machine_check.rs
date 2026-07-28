// SPDX-License-Identifier: GPL-3.0-only
//! Machine-check exception handler (vector 18, `#MC`).
//!
//! Known limitation: `kernel_panic` takes `IrqMutex` locks, so an MCE fired while
//! one is held can deadlock (TODO #21).
//!
//! Authors: MarioS271

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the interrupt stack frame. Never returns.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame
) -> ! {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(&mut fmt_buffer, "{:#?}\n", isf);
    kernel_panic(
        PanicCode::MachineCheck,
        fmt_buffer.as_str(),
    );
}
