//! Kernel print macro and unified logging output.
//!
//! [`kprint!`] is the primary logging interface for the kernel. It accepts the same
//! format string syntax as `print!`. Each call goes through [`KernelWriter`], which
//! holds an [`IrqMutexGuard`] for its entire lifetime. That guard is acquired once
//! when the macro creates the [`KernelWriter`] via [`KernelWriter::lock`], and is
//! held across all `write_str` fragments that [`core::fmt::write`] may issue for a
//! single format string. This prevents an IRQ handler's `kprint!` from interleaving
//! between fragments of an in-progress call.
//!
//! Each `write_str` call:
//!
//! 1. Appends the text to an in-memory ring buffer (`log_buf`) so the log persists
//!    in RAM even if serial or framebuffer output was not yet ready.
//! 2. Writes to serial (if initialized).
//! 3. Renders to the framebuffer (if initialized), with software scrolling: when
//!    the cursor reaches the last row, the framebuffer contents are shifted up by
//!    one glyph height using `core::ptr::copy`, and the bottom row is cleared.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::types::irq_mutex::{IrqMutex, IrqMutexGuard};

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

fn kprint(state: &mut IrqMutexGuard<'static, KernelPrintState>, string: &str) {
    use crate::SIMPLE_STATE;

    write_log_buf(state, string);

    if SIMPLE_STATE.serial.is_completed() {
        use crate::logging::_serial::_Serial;
        SIMPLE_STATE.serial.get().unwrap().write(string);
    }

    let basic_fb = &SIMPLE_STATE.basic_fb;
    let basic_fb_psf2_font = &SIMPLE_STATE.basic_fb_psf2_font;

    if basic_fb.is_completed() && basic_fb_psf2_font.is_completed() {
        let fb = basic_fb.get().unwrap();
        let font = basic_fb_psf2_font.get().unwrap();

        let s = &mut **state;
        font.draw_string(fb, string, &mut s.cursor_x, &mut s.cursor_y, None);

        let last_row = fb.height as usize - font.glyph_height();
        if s.cursor_y > last_row {
            s.cursor_y = last_row;

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

/// RAII handle that holds the [`KPRINT_STATE`] lock for the duration of a `kprint!` call.
///
/// Constructed by [`KernelWriter::lock`], which acquires [`KPRINT_STATE`] and stores
/// the guard as a field. [`core::fmt::write`] then calls [`write_str`] one or more
/// times on this handle; each call borrows the already-held guard rather than
/// acquiring and releasing the lock per fragment. When the handle drops at the end
/// of the `kprint!` macro block, the guard drops with it, releasing the lock and
/// restoring the interrupt flag.
///
/// [`write_str`]: core::fmt::Write::write_str
pub struct KernelWriter {
    lock: IrqMutexGuard<'static, KernelPrintState>
}
impl KernelWriter {
    /// Acquire [`KPRINT_STATE`] and return a [`KernelWriter`] that holds the guard.
    ///
    /// Disables interrupts for the duration of the returned handle's lifetime.
    /// Called once per `kprint!` invocation; do not call directly.
    pub fn lock() -> Self {
        Self { lock: KPRINT_STATE.lock() }
    }
}
impl core::fmt::Write for KernelWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        kprint(&mut self.lock, string);
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
            &mut $crate::logging::kprint::KernelWriter::lock(),
            format_args!($($arg)*)
        );
    });
}
