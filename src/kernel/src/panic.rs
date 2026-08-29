// SPDX-License-Identifier: GPL-3.0-only
//! Kernel panic paths: [`kernel_panic`] for explicit kernel panics and the
//! required `#[panic_handler]` for language-level panics.
//!
//! Authors: MarioS271

use crate::arch::instructions;
use crate::types::fmt_buffer::FmtBuffer;
use crate::types::panic_codes::PanicCode;
use crate::SIMPLE_STATE;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

/// Prevents infinite panic re-entry on the `kernel_panic` path.
static KERNEL_PANIC_TRIGERRED: AtomicBool = AtomicBool::new(false);

/// Prevents infinite panic re-entry on the `#[panic_handler]` path.
static RUST_PANIC_TRIGERRED: AtomicBool = AtomicBool::new(false);

/// Halt the kernel with a diagnostic message; re-entrant calls skip straight to the halt loop.
#[cold]
pub fn kernel_panic(panic_code: PanicCode, panic_message: &str) -> ! {
    instructions::disable_interrupts();
    force_unlock_loggers();

    if KERNEL_PANIC_TRIGERRED.load(Ordering::Acquire) {
        loop {
            instructions::halt_cpu();
        }
    }
    KERNEL_PANIC_TRIGERRED.store(true, Ordering::Release);

    {  // todo: add init check for serial
        use crate::logging::serial::_Serial;
        let serial = SIMPLE_STATE.serial().lock();

        serial.write("Kernel Panic! :(\n");
        serial.write(panic_code.as_str());
        serial.write("\n");
        serial.write(panic_message);
    }

    {  // todo: add init check for basic fb and psf2 font
        let fb = SIMPLE_STATE.basic_fb().lock();
        let font = SIMPLE_STATE.basic_fb_psf2_font();
        let mut x: usize = 0;
        let mut y: usize = 0;

        fb.clear();
        font.draw_string(&*fb, "Kernel Panic! :(\n", &mut x, &mut y, Some(0x00FF0000));
        font.draw_string(&*fb, panic_code.as_str(), &mut x, &mut y, None);
        font.draw_string(&*fb, "\n", &mut x, &mut y, None);
        font.draw_string(&*fb, panic_message, &mut x, &mut y, None);
    }

    loop {
        instructions::halt_cpu();
    }
}

/// Rust's required `#[panic_handler]` for language-level panics.
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    use core::fmt::Write;

    instructions::disable_interrupts();
    force_unlock_loggers();

    if RUST_PANIC_TRIGERRED.load(Ordering::Acquire) {
        loop {
            instructions::halt_cpu();
        }
    }
    RUST_PANIC_TRIGERRED.store(true, Ordering::Release);

    let mut message_buf: FmtBuffer<512> = FmtBuffer::new();
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

    {  // todo: add init check for serial
        use crate::logging::serial::_Serial;
        let serial = SIMPLE_STATE.serial().lock();

        serial.write("\nKernel Panic! :(\n");
        serial.write("[!] This panic was triggered by Rust (runtime error or panic!() called)\n\n");
        serial.write(message_buf.as_str());
        serial.write(location_buf.as_str());
    }

    {  // todo: add init check for basic fb and psf2 font
        let fb = SIMPLE_STATE.basic_fb().lock();
        let font = SIMPLE_STATE.basic_fb_psf2_font();
        let mut x: usize = 0;
        let mut y: usize = 0;

        fb.clear();
        font.draw_string(&*fb, "Kernel Panic! :(\n", &mut x, &mut y, Some(0x00FF0000));
        font.draw_string(&*fb, "[!] This panic was triggered by Rust (runtime error or panic!() called)\n\n", &mut x, &mut y, None);
        font.draw_string(&*fb, message_buf.as_str(), &mut x, &mut y, None);
        font.draw_string(&*fb, location_buf.as_str(), &mut x, &mut y, None);
    }

    loop {
        instructions::halt_cpu();
    }
}

/// Force-unlocks the kernel serial logger and the basic framebuffer
///
/// # Safety
/// Worst case, serial + basic fb output gets interrupted/cut off, which isn't a big deal
/// considering the kernel has panicked
fn force_unlock_loggers() {
    // Safety: refer to above notice
    unsafe {
        SIMPLE_STATE.serial().force_unlock();
        SIMPLE_STATE.basic_fb().force_unlock();
    }
}
