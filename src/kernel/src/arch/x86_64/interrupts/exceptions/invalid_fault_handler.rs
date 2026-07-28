//! Generic handler for interrupt vectors that cannot legally fire in 64-bit long mode.
//!
//! Certain x86 exception vectors (e.g., #5 bound-range exceeded, #9 coprocessor
//! segment overrun) are architecturally impossible in 64-bit mode. This handler is
//! registered for those slots so that if one fires anyway — indicating IDT corruption
//! or a CPU errata — it produces a diagnostic panic rather than jumping to address 0.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::fmt::Write;
use x86_64::structures::idt::InterruptStackFrame;
use crate::types::fmt_buffer::FmtBuffer;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic with the vector number and interrupt stack frame.
///
/// The `VEC` const generic carries the IDT vector index so the panic message can
/// identify which impossible interrupt fired. The ISF is included to aid diagnosis.
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
