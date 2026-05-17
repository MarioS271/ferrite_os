//! panic.rs
//! Panic Handler
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::types::panic_codes::PanicCode;
use crate::screen::basic;
use core::arch::asm;
use core::panic::PanicInfo;

/// Kernel's custom panic handler, prints debug text and halts
pub fn kernel_panic(panic_code: PanicCode, panic_message: &str, print_debug_text: bool) -> ! {
    // Necessary to disable any interrupts to prevent the panic sequence from being interrupted
    unsafe {
        asm!("cli", options(nostack, nomem))
    }

    if print_debug_text {
        if let Some(fb) = basic::framebuffer::get_framebuffer() {
            let mut x: usize = 0;
            let mut y: usize = 0;

            basic::text::draw_string(fb, "Panic!\n", &mut x, &mut y, Some(0x00FF0000));
            basic::text::draw_string(fb, panic_code.as_str(), &mut x, &mut y, None);
            basic::text::draw_string(fb, "\n", &mut x, &mut y, None);
            basic::text::draw_string(fb, panic_message, &mut x, &mut y, None);
        }
    }

    loop {
        // Halt the cpu cause the system is gone
        unsafe {
            asm!("hlt", options(nostack, nomem))
        }
    }
}

/// Rust's internal panic handler, only used when rust runtime faults occur
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        // Necessary to disable any interrupts to prevent the panic sequence from being interrupted,
        // and halt the cpu cause the system is gone
        unsafe {
            asm!("cli; hlt", options(nostack, nomem))
        }
    }
}
