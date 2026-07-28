//! Non-Maskable Interrupt (NMI) handler (vector 2).
//!
//! An NMI cannot be masked by the CPU's IF flag and typically indicates a hardware
//! failure such as a memory parity error or a watchdog timeout. It runs on IST
//! stack 2 so it can fire safely regardless of the main stack's state. Currently
//! treated as a fatal hardware failure.
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
        PanicCode::NmiHardwareFailiure,
        fmt_buffer.as_str(),
    );
}
