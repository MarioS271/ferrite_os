//! arch/exceptions/general_protection_fault.rs
//! General Protection Fault Exception Handler
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::kprint;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(
    interrupt_stack_frame: InterruptStackFrame, error_code: u64
) {
    kprint!("\n\nGeneral Protection Fault!\n");
    kprint!("Error Code: {error_code}\n");
    kprint!("{interrupt_stack_frame:#?}\n");
    kernel_panic(
        PanicCode::GeneralProtectionFault,
        "",
        true
    );
}