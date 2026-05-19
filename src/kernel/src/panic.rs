//! panic.rs
//! Panic Handler
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::types::panic_codes::PanicCode;
use core::arch::asm;
use core::panic::PanicInfo;
use crate::screen::basic::framebuffer::clear_framebuffer;

/// Kernel's custom panic handler, prints debug text and halts
pub fn kernel_panic(panic_code: PanicCode, panic_message: &str, print_debug_text: bool) -> ! {
    // Necessary to disable any interrupts to prevent the panic sequence from being interrupted
    unsafe {
        asm!("cli", options(nostack, nomem))
    }

    if print_debug_text {
        if let Some(fb) = crate::screen::basic::framebuffer::get_framebuffer() {
            let mut x: usize = 0;
            let mut y: usize = 0;

            use crate::screen::basic::framebuffer::clear_framebuffer;
            use crate::screen::basic::text::draw_string;

            clear_framebuffer(fb);
            draw_string(fb, "Kernel Panic!\n", &mut x, &mut y, Some(0x00FF0000));
            draw_string(fb, panic_code.as_str(), &mut x, &mut y, None);
            draw_string(fb, "\n", &mut x, &mut y, None);
            draw_string(fb, panic_message, &mut x, &mut y, None);
        }
    }

    use crate::logging::serial::write_string_to_com1;

    write_string_to_com1("\nKernel Panic!\n");
    write_string_to_com1(panic_code.as_str());
    write_string_to_com1("\n");
    write_string_to_com1(panic_message);

    loop {
        // Halt the cpu cause the system is gone
        unsafe {
            asm!("hlt", options(nostack, nomem))
        }
    }
}

/// Rust's internal panic handler, only used when rust runtime faults occur
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    // Necessary to disable any interrupts to prevent the panic sequence from being interrupted
    unsafe {
        asm!("cli", options(nostack, nomem))
    }

    use crate::logging::serial::write_string_to_com1;

    write_string_to_com1("\nFATAL: rustlang panic handler fired!\n");
    write_string_to_com1(
        panic_info.message().as_str().unwrap_or("(No panic info message given)")
    );

    loop {
        // Halt the cpu cause the system is gone
        unsafe {
            asm!("hlt", options(nostack, nomem))
        }
    }
}
