//! arch/exceptions/double_fault.rs
//! Double Fault Exception Handler
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::kprint;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(
    interrupt_stack_frame: InterruptStackFrame, _error_code: u64
) -> ! {
    kprint!("\n\nDouble Fault!\n");
    kprint!("{interrupt_stack_frame:#?}\n");
    kernel_panic(
        PanicCode::DoubleFault,
        "",
        true
    );
}