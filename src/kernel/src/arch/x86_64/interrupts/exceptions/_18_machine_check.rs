//! Machine-check exception handler (vector 18, `#MC`).
//!
//! Fires when the CPU's Machine Check Architecture (MCA) detects an unrecoverable
//! hardware error such as a memory ECC fault or bus error. Runs on IST stack 3
//! and is always diverging (the CPU model requires a non-returning handler).
//!
//! **Known limitation**: `kernel_panic` acquires `IrqMutex` locks internally; an
//! MCE can fire while any of those locks are held, potentially causing a deadlock.
//! A future fix should bypass all locks and write directly to serial (TODO #21).
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

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
