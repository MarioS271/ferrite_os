//! Kernel print macro and unified logging output.
//!
//! [`kprint!`] is the primary logging interface for the kernel. It accepts the same
//! format string syntax as `print!`. Each call goes through [`KernelWriter`] which
//! calls the private `kprint` function. That function:
//!
//! 1. Appends the text to an in-memory ring buffer (`log_buf`) so the log persists
//!    in RAM even if serial or framebuffer output was not yet ready.
//! 2. Writes to serial (if initialized).
//! 3. Renders to the framebuffer (if initialized), with software scrolling: when
//!    the cursor reaches the last row, the framebuffer contents are shifted up by
//!    one glyph height using `core::ptr::copy`, and the bottom row is cleared.
//!
//! The entire operation is protected by [`KPRINT_STATE`]'s [`IrqMutex`], so
//! `kprint!` is safe to call from interrupt handlers.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::types::irq_mutex::IrqMutex;

/// Capacity of the in-memory log ring buffer in bytes.
const LOG_BUFFER_SIZE: usize = u16::MAX as usize;

/// All mutable state for the `kprint` subsystem, protected by an [`IrqMutex`].
struct KernelPrintState {
    /// Circular byte buffer holding the most recent log output.
    log_buf: [u8; LOG_BUFFER_SIZE],
    /// Index of the next byte to write. Wraps via modulo `LOG_BUFFER_SIZE`.
    log_head: usize,
    /// Index of the oldest readable byte. Advances when `log_head` laps it.
    log_tail: usize,
    /// Current framebuffer cursor column in pixels.
    cursor_x: usize,
    /// Current framebuffer cursor row in pixels.
    cursor_y: usize,
}

/// Global logging state. Wrapped in [`IrqMutex`] so interrupt handlers can log safely.
static KPRINT_STATE: IrqMutex<KernelPrintState> = IrqMutex::new(KernelPrintState {
    log_buf: [0u8; LOG_BUFFER_SIZE],
    log_head: 0,
    log_tail: 0,
    cursor_x: 0,
    cursor_y: 0,
});

fn write_log_buf(state: &mut KernelPrintState, s: &str) {
    for &b in s.as_bytes() {
        state.log_buf[state.log_head % LOG_BUFFER_SIZE] = b;
        state.log_head = state.log_head.wrapping_add(1);
        if state.log_head.wrapping_sub(state.log_tail) > LOG_BUFFER_SIZE {
            state.log_tail = state.log_tail.wrapping_add(1);
        }
    }
}

fn kprint(string: &str) {
    use crate::SIMPLE_STATE;

    let mut state = KPRINT_STATE.lock();

    write_log_buf(&mut state, string);

    if SIMPLE_STATE.serial.is_completed() {
        use crate::logging::_serial::_Serial;
        SIMPLE_STATE.serial.get().unwrap().write(string);
    }

    let basic_fb = &SIMPLE_STATE.basic_fb;
    let basic_fb_psf2_font = &SIMPLE_STATE.basic_fb_psf2_font;

    if basic_fb.is_completed() && basic_fb_psf2_font.is_completed() {
        let fb = basic_fb.get().unwrap();
        let font = basic_fb_psf2_font.get().unwrap();

        let s = &mut *state;
        font.draw_string(fb, string, &mut s.cursor_x, &mut s.cursor_y, None);

        let last_row = fb.height as usize - font.glyph_height();
        if state.cursor_y > last_row {
            state.cursor_y = last_row;

            // Safe: src and dst are within framebuffer bounds, counts derived from fb dimensions
            unsafe {
                let src = fb.fb_pointer.add(font.glyph_height() * fb.bytes_per_row as usize);
                let dst = fb.fb_pointer;
                let count = (fb.height as usize - font.glyph_height()) * fb.bytes_per_row as usize;
                core::ptr::copy(src, dst, count);
            }

            // Safe: iteration range is within framebuffer bounds
            unsafe {
                let start = (fb.height as usize - font.glyph_height()) * fb.bytes_per_row as usize;
                let end = fb.height as usize * fb.bytes_per_row as usize;
                for i in start..end {
                    fb.fb_pointer.add(i).write_volatile(0u32);
                }
            }
        }
    }
}

/// Zero-sized type that implements [`core::fmt::Write`] by forwarding to `kprint`.
///
/// Used by the [`kprint!`] macro: `core::fmt::write` takes a `&mut dyn Write`
/// and calls `write_str` one or more times with formatted fragments.
pub struct KernelWriter;
impl core::fmt::Write for KernelWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        kprint(string);
        Ok(())
    }
}

/// Print a formatted message to all active kernel output sinks.
///
/// Accepts the same format string syntax as `std::print!`. Output goes to the
/// in-memory log ring buffer, the serial port (if initialized), and the framebuffer
/// (if initialized, with automatic scrolling). Safe to call from interrupt handlers.
///
/// # Examples
/// ```ignore
/// kprint!("Hello, {}!\n", "FerriteOS");
/// kprint!("[PMM] total_frames={}\n", count);
/// ```
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ({
        let _ = core::fmt::write(
            &mut $crate::logging::kprint::KernelWriter,
            format_args!($($arg)*)
        );
    });
}
