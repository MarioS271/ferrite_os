// SPDX-License-Identifier: GPL-3.0-only
//! Kernel panic paths: [`kernel_panic`] for explicit kernel panics and the
//! required `#[panic_handler]` for language-level panics.
//!
//! Authors: MarioS271

use crate::cpu::instructions;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::fmt_buffer::FmtBuffer;
use crate::SIMPLE_STATE;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU8, Ordering};

// TODO: respect SIMPLE_STATE.panic_action instead of hardcoded halt

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

    if PANIC_TRIGGERED.load(Ordering::Acquire) & PanicTriggered::WasKernelPanicTriggered as u8 != 0 {
        core::hint::cold_path();
        loop {
            instructions::halt_cpu();
        }
    }
    PANIC_TRIGGERED.fetch_or(PanicTriggered::WasKernelPanicTriggered as u8, Ordering::AcqRel);

    // Safety: We acknowledge the risk of corrupted output, which isn't too bad considering we already panic'd
    unsafe { force_unlock_loggers(); }

    if SIMPLE_STATE.is_uart_initialized() {
        use crate::drivers::tty::uart::UartOps;

        // Safety: we check above if uart is already initialized, making this safe
        let mut uart = unsafe { SIMPLE_STATE.uart().lock() };

        let _ = uart.write_string("\nKernel Panic! :(\n");
        let _ = uart.write_string("Panic Code: ");
        let _ = uart.write_string(panic_code.as_str());
        let _ = uart.write_string("\nPanic Type: ");
        let _ = uart.write_string(panic_code.get_error_type_str());
        let _ = uart.write_string("\n\n");
        let _ = uart.write_string(panic_message);
    }

    if SIMPLE_STATE.is_basic_fb_initialized() && SIMPLE_STATE.is_basic_fb_psf2_font_initialized() {
        // Safety: we check above if basic_fb is already initialized, making this safe
        let fb = unsafe { SIMPLE_STATE.basic_fb().lock() };
        // Safety: we check above if basic_fb_psf2_font is already initialized, making this safe
        let font = unsafe { SIMPLE_STATE.basic_fb_psf2_font() };

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

    instructions::halt_forever();
}

/// Rust's required `#[panic_handler]` for language-level panics.
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    use core::fmt::Write;

    instructions::disable_interrupts();

    if PANIC_TRIGGERED.load(Ordering::Acquire) & PanicTriggered::WasRustPanicTriggered as u8 != 0 {
        core::hint::cold_path();
        loop {
            instructions::halt_cpu();
        }
    }
    PANIC_TRIGGERED.fetch_or(PanicTriggered::WasRustPanicTriggered as u8, Ordering::AcqRel);

    // Safety: We acknowledge the risk of corrupted output, which isn't too bad considering we already panic'd
    unsafe { force_unlock_loggers(); }

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

    if SIMPLE_STATE.is_uart_initialized() {
        use crate::drivers::tty::uart::UartOps;

        // Safety: we check above if uart is already initialized, making this safe
        let mut uart = unsafe { SIMPLE_STATE.uart().lock() };

        let _ = uart.write_string("\nKernel Panic! :(\n");
        let _ = uart.write_string("[!] This panic was triggered by Rust (runtime error or panic!() called)\n\n");
        let _ = uart.write_string(message_buf.as_str());
        let _ = uart.write_string(location_buf.as_str());
    }

    if SIMPLE_STATE.is_basic_fb_initialized() && SIMPLE_STATE.is_basic_fb_psf2_font_initialized() {
        // Safety: we check above if basic_fb is already initialized, making this safe
        let fb = unsafe { SIMPLE_STATE.basic_fb().lock() };
        // Safety: we check above if basic_fb_psf2_font is already initialized, making this safe
        let font = unsafe { SIMPLE_STATE.basic_fb_psf2_font() };

        let mut x: usize = 0;
        let mut y: usize = 0;

        fb.clear();
        font.draw_string(&*fb, "Kernel Panic! :(\n", &mut x, &mut y, Some(0x00FF0000));
        font.draw_string(&*fb, "[!] This panic was triggered by Rust (runtime error or panic!() called)\n\n", &mut x, &mut y, None);
        font.draw_string(&*fb, message_buf.as_str(), &mut x, &mut y, None);
        font.draw_string(&*fb, location_buf.as_str(), &mut x, &mut y, None);
    }

    instructions::halt_forever();
}

/// Force-unlocks the kernel serial logger and the basic_fb framebuffer
///
/// # Safety
/// Calling this will force-unlock the serial and basic_fb `IrqMutex`es, which can result in
/// corrupted output. This should only be called from irrecoverable scenarios such as a kernel panic.
unsafe fn force_unlock_loggers() {
    unsafe {
        if SIMPLE_STATE.is_uart_initialized() {
            SIMPLE_STATE.uart().force_unlock();
        }
        if SIMPLE_STATE.is_basic_fb_initialized() && SIMPLE_STATE.is_basic_fb_psf2_font_initialized() {
            SIMPLE_STATE.basic_fb().force_unlock();
        }
    }
}
