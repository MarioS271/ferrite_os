//! panic.rs
//! Panic Handler
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::types::panic_codes::PanicCode;
use crate::arch::instructions;
use crate::SIMPLE_STATE;
use crate::types::fmt_buffer::FmtBuffer;

static KERNEL_PANIC_TRIGERRED: AtomicBool = AtomicBool::new(false);
static RUST_PANIC_TRIGERRED: AtomicBool = AtomicBool::new(false);

/// Kernel's custom panic handler, prints debug text and halts
pub fn kernel_panic(panic_code: PanicCode, panic_message: &str) -> ! {
    instructions::disable_interrupts();

    if KERNEL_PANIC_TRIGERRED.load(Ordering::Acquire) {
        loop {
            instructions::halt_cpu();
        }
    }
    KERNEL_PANIC_TRIGERRED.store(true, Ordering::Release);

    if SIMPLE_STATE.serial.is_completed() {
        use crate::logging::_serial::_Serial;
        let serial = SIMPLE_STATE.serial.get().unwrap();

        serial.write("Kernel Panic! :(\n");
        serial.write(panic_code.as_str());
        serial.write("\n");
        serial.write(panic_message);
    }

    if SIMPLE_STATE.basic_fb.is_completed() && SIMPLE_STATE.basic_fb_psf2_font.is_completed() {
        let fb = SIMPLE_STATE.basic_fb.get().unwrap();
        let font = SIMPLE_STATE.basic_fb_psf2_font.get().unwrap();
        let mut x: usize = 0;
        let mut y: usize = 0;

        fb.clear();
        font.draw_string(fb, "Kernel Panic! :(\n", &mut x, &mut y, Some(0x00FF0000));
        font.draw_string(fb, panic_code.as_str(), &mut x, &mut y, None);
        font.draw_string(fb, "\n", &mut x, &mut y, None);
        font.draw_string(fb, panic_message, &mut x, &mut y, None);
    }

    loop {
        instructions::halt_cpu();
    }
}

/// Rust's internal panic handler, only used when rust runtime exceptions occur
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    use core::fmt::Write;

    instructions::disable_interrupts();

    if RUST_PANIC_TRIGERRED.load(Ordering::Acquire) {
        loop {
            instructions::halt_cpu();
        }
    }
    RUST_PANIC_TRIGERRED.store(true, Ordering::Release);

    let mut message_buf: FmtBuffer<128> = FmtBuffer::new();
    if panic_info.message().as_str().is_none() {
        let _ = write!(message_buf, "(No panic message given)\n\n");
    } else {
        let _ = write!(message_buf, "Panic Message: {}\n\n", panic_info.message());
    }

    let mut location_buf: FmtBuffer<256> = FmtBuffer::new();
    if panic_info.location().is_none() {
        let _ = write!(location_buf, "(No panic location given)");
    } else {
        let location = panic_info.location().unwrap();
        let _ = write!(location_buf, "{}\n  on line {}", location.file(), location.line());
    }

    if SIMPLE_STATE.serial.is_completed() {
        use crate::logging::_serial::_Serial;
        let serial = SIMPLE_STATE.serial.get().unwrap();

        serial.write("\nKernel Panic! :(\n");
        serial.write("[!] This panic was triggered by Rust (language-triggered or via a panic!() call)\n\n");
        serial.write(message_buf.as_str());
        serial.write(location_buf.as_str());
    }

    if SIMPLE_STATE.basic_fb.is_completed() && SIMPLE_STATE.basic_fb_psf2_font.is_completed() {
        let fb = SIMPLE_STATE.basic_fb.get().unwrap();
        let font = SIMPLE_STATE.basic_fb_psf2_font.get().unwrap();
        let mut x: usize = 0;
        let mut y: usize = 0;

        fb.clear();
        font.draw_string(fb, "Kernel Panic! :(\n", &mut x, &mut y, Some(0x00FF0000));
        font.draw_string(fb, "[!] This panic was triggered by Rust (runtime error or panic!() called)\n\n", &mut x, &mut y, None);
        font.draw_string(fb, message_buf.as_str(), &mut x, &mut y, None);
        font.draw_string(fb, location_buf.as_str(), &mut x, &mut y, None);
    }

    loop {
        instructions::halt_cpu();
    }
}
