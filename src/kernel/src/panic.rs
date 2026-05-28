//! panic.rs
//! Panic Handler
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::types::panic_codes::PanicCode;
use crate::logging::serial::write_to_serial;
use core::arch::asm;
use core::panic::PanicInfo;
use crate::arch;

/// Kernel's custom panic handler, prints debug text and halts
pub fn kernel_panic(panic_code: PanicCode, panic_message: &str, print_debug_text: bool) -> ! {
    arch::instructions::disable_interrupts();

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

    write_to_serial("\nKernel Panic!\n");
    write_to_serial(panic_code.as_str());
    write_to_serial("\n");
    write_to_serial(panic_message);

    loop {
        arch::instructions::halt_cpu();
    }
}

/// Rust's internal panic handler, only used when rust runtime faults occur
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    arch::instructions::disable_interrupts();

    write_to_serial("\nFATAL: rustlang panic handler fired!\n");
    write_to_serial(
        panic_info.message().as_str().unwrap_or("(No panic info message given)")
    );

    loop {
        arch::instructions::halt_cpu();
    }
}
