//! panic.rs
//! Panic Handler
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::panic::PanicInfo;
use crate::arch::instructions;
use crate::types::panic_codes::PanicCode;
use crate::SIMPLE_STATE;
use crate::types::fmt_buffer::FmtBuffer;

const PANIC_HEADER: &str = "\nKernel Panic! :(\n";

/// Kernel's custom panic handler, prints debug text and halts
pub fn kernel_panic(panic_code: PanicCode, panic_message: &str) -> ! {
    instructions::disable_interrupts();

    if SIMPLE_STATE.serial.is_completed() {
        use crate::logging::_serial::_Serial;
        let serial = SIMPLE_STATE.serial.get().unwrap();

        serial.write(PANIC_HEADER);
        serial.write(panic_code.as_str());
        serial.write("\n");
        serial.write(panic_message);
    }

    if SIMPLE_STATE.basic_fb.is_completed() {
        if let Some(fb) = crate::screen::basic::framebuffer::get_framebuffer() {
            let mut x: usize = 0;
            let mut y: usize = 0;

            use crate::screen::basic::text::draw_string;

            fb.clear();
            draw_string(fb, PANIC_HEADER, &mut x, &mut y, Some(0x00FF0000));
            draw_string(fb, panic_code.as_str(), &mut x, &mut y, None);
            draw_string(fb, "\n", &mut x, &mut y, None);
            draw_string(fb, panic_message, &mut x, &mut y, None);
        }
    }

    loop {
        instructions::halt_cpu();
    }
}

/// Rust's internal panic handler, only used when rust runtime exceptions occur
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    instructions::disable_interrupts();

    if SIMPLE_STATE.serial.is_completed() {
        use core::fmt::Write;
        use crate::logging::_serial::_Serial;

        let serial = SIMPLE_STATE.serial.get().unwrap();

        let mut message_buf: FmtBuffer<32> = FmtBuffer::new();
        if panic_info.message().as_str().is_none() {
            let _ = write!(message_buf, "(No panic message given)\n\n");
        }
        else {
            let _ = write!(message_buf, "Panic Message: {}\n\n", panic_info.message());
        }

        serial.write(PANIC_HEADER);
        serial.write("[!] This panic was triggered by Rust (language-triggered or via a panic!() call)\n");
        serial.write(message_buf.as_str());

        if panic_info.location().is_none() {
            serial.write("(No panic location given)");
        } else {
            let location = panic_info.location().unwrap();
            let mut line_num_buf: FmtBuffer<32> = FmtBuffer::new();
            let _ = write!(line_num_buf, "  on line {}", location.line());

            serial.write(location.file());
            serial.write("\n");
            serial.write(line_num_buf.as_str());
        }
    }

    loop {
        instructions::halt_cpu();
    }
}
