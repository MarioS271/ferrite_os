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
use core::sync::atomic::{AtomicU8, Ordering};

/// Used to prevent infinite panic handler reentry
static PANIC_TRIGGERED: AtomicU8 = AtomicU8::new(0);

/// Enum helper to not have to write the raw bitshifts on every PANIC_TRIGGERED interaction
#[repr(u8)]
enum PanicTriggered {
    WasKernelPanicTriggered = 1 << 0,
    WasRustPanicTriggered = 1 << 1
}

/// Halt the kernel with a diagnostic message; re-entrant calls skip straight to the halt loop.
#[cold]
pub fn kernel_panic(panic_code: PanicCode, panic_message: &str) -> ! {
    instructions::disable_interrupts();
    force_unlock_loggers();

    if PANIC_TRIGGERED.load(Ordering::Acquire) & PanicTriggered::WasKernelPanicTriggered as u8 != 0 {
        core::hint::cold_path();
        loop {
            instructions::halt_cpu();
        }
    }
    PANIC_TRIGGERED.fetch_or(PanicTriggered::WasKernelPanicTriggered as u8, Ordering::AcqRel);

    if SIMPLE_STATE.is_serial_initialized() {
        use crate::logging::serial::_Serial;
        let serial = SIMPLE_STATE.serial().lock();

        serial.write("\nKernel Panic! :(\n");
        serial.write("Panic Code: ");
        serial.write(panic_code.as_str());
        serial.write("\nPanic Type: ");
        serial.write(panic_code.get_error_type_str());
        serial.write("\n\n");
        serial.write(panic_message);
    }

    if SIMPLE_STATE.is_basic_fb_initialized() && SIMPLE_STATE.is_basic_fb_psf2_font_initialized() {
        let fb = SIMPLE_STATE.basic_fb().lock();
        let font = SIMPLE_STATE.basic_fb_psf2_font();
        let mut x: usize = 0;
        let mut y: usize = 0;

        fb.clear();
        font.draw_string(&*fb, "Kernel Panic! :(\n", &mut x, &mut y, Some(0x00FF0000));
        font.draw_string(&*fb, "Panic Code: ", &mut x, &mut y, None);
        font.draw_string(&*fb, panic_code.as_str(), &mut x, &mut y, None);
        font.draw_string(&*fb, "\nPanic Type: ", &mut x, &mut y, None);
        font.draw_string(&*fb, panic_code.get_error_type_str(), &mut x, &mut y, None);
        font.draw_string(&*fb, "\n\n", &mut x, &mut y, None);
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

    if PANIC_TRIGGERED.load(Ordering::Acquire) | PanicTriggered::WasRustPanicTriggered as u8 != 0 {
        core::hint::cold_path();
        loop {
            instructions::halt_cpu();
        }
    }
    PANIC_TRIGGERED.fetch_or(PanicTriggered::WasRustPanicTriggered as u8, Ordering::AcqRel);

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

    if SIMPLE_STATE.is_serial_initialized() {
        use crate::logging::serial::_Serial;
        let serial = SIMPLE_STATE.serial().lock();

        serial.write("\nKernel Panic! :(\n");
        serial.write("[!] This panic was triggered by Rust (runtime error or panic!() called)\n\n");
        serial.write(message_buf.as_str());
        serial.write(location_buf.as_str());
    }

    if SIMPLE_STATE.is_basic_fb_initialized() && SIMPLE_STATE.is_basic_fb_psf2_font_initialized() {
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
