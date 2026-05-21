//! arch/exceptions/page_fault.rs
//! Page Fault Exception Handler
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::kprint;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;

pub extern "x86-interrupt" fn handler(
    interrupt_stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode
) {
    let cr2_value = Cr2::read_raw();

    kprint!("\n\nPage Fault!\n");
    kprint!("CR2 Value: {cr2_value}");
    kprint!("Error Code:\n");
    kprint!("{error_code:?}");
    kprint!("{interrupt_stack_frame:#?}\n");
    kernel_panic(
        PanicCode::PageFault,
        "",
        true
    );
}