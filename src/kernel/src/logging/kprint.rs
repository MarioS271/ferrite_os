// SPDX-License-Identifier: GPL-3.0-only
//! Kernel print macro and unified logging output.
//!
//! [`kprint!`] is the primary logging interface for the kernel. Each call holds an
//! [`IrqMutexGuard`] for its entire duration, preventing IRQ handlers from interleaving
//! output mid-format. Output goes to an in-memory ring buffer, serial, and the framebuffer
//! (with software scrolling when the cursor reaches the last row).
//!
//! Authors: MarioS271

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
    cursor_x: usize,
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

/// Write `string` to the ring buffer, serial port, and framebuffer (if each is initialized).
fn kprint(state: &mut IrqMutexGuard<'static, KernelPrintState>, string: &str, color: Option<u32>) {
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
        font.draw_string(fb, string, &mut s.cursor_x, &mut s.cursor_y, color);

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

/// Release [`KPRINT_STATE`]'s lock without restoring the interrupt flag.
///
/// Thin wrapper around [`IrqMutex::force_unlock`] for use in panic paths that cannot
/// access [`KPRINT_STATE`] directly. See [`IrqMutex::force_unlock`] for the full
/// safety contract.
///
/// # Safety
/// Same as [`IrqMutex::force_unlock`]: must only be called from a panic handler that
/// will halt the CPU immediately after and will not access [`KPRINT_STATE`] through
/// the mutex again.
pub unsafe fn force_unlock_kprint_state() {
    KPRINT_STATE.force_unlock();
}

/// RAII handle that holds the [`KPRINT_STATE`] lock for the duration of a `kprint!` call.
/// Holds the lock across all `write_str` fragments so output cannot be interleaved.
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

    pub fn print_raw(&mut self, string: &str, color: Option<u32>) {
        kprint(&mut self.lock, string, color);
    }
}
impl core::fmt::Write for KernelWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        kprint(&mut self.lock, string, None);
        Ok(())
    }
}

/// Print a formatted message to all active kernel output sinks.
///
/// Accepts the same format string syntax as `std::print!`. Output goes to the
/// in-memory log ring buffer, the serial port (if initialized), and the framebuffer
/// (if initialized, with automatic scrolling). Safe to call from interrupt handlers.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            let _ = core::fmt::write(
                &mut $crate::logging::kprint::KernelWriter::lock(),
                format_args!($($arg)*)
            );
        }
    };
}

pub enum LogLevelColor {
    Emergency = 0x00ff00aa,
    Alert = 0x00ffaa00,
    Critical = 0x008a00ff,
    Error = 0x00ff0000,
    Warn = 0x00ffff00,
    Info = 0x0000ff00,
    Debug = 0x000000ff,
}

#[macro_export]
macro_rules! kemerg {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_raw("! ", Some(LogLevelColor::Emergency as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
#[macro_export]
macro_rules! kalert {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_raw("A ", Some(LogLevelColor::Alert as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
#[macro_export]
macro_rules! kcrit {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_raw("C ", Some(LogLevelColor::Critical as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_raw("E ", Some(LogLevelColor::Error as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_raw("W ", Some(LogLevelColor::Warn as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_raw("I ", Some(LogLevelColor::Info as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-logging")]
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_raw("D ", Some(LogLevelColor::Debug as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
