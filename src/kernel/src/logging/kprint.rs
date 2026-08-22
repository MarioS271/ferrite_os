// SPDX-License-Identifier: GPL-3.0-only
//! Kernel print macro ([`kprint!`]) and the logging state behind it.
//!
//! Authors: MarioS271

use crate::types::irq_mutex::{IrqMutex, IrqMutexGuard};

/// Capacity of the in-memory log ring buffer (default: 512 KiB).
const LOG_BUFFER_SIZE: usize = 1 << 19;

/// Marker which signifies that the next byte is a log level indicator byte. ("!", "A", "C", ...)
const LOG_LEVEL_MARKER: &str = "\x01";

/// All mutable state for the `kprint` subsystem, protected by an [`IrqMutex`].
struct KernelPrintState {
    log_buf: [u8; LOG_BUFFER_SIZE],
    log_head: usize,
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

/// Append `s` to the ring buffer, advancing the tail when the head laps it.
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

/// Release [`KPRINT_STATE`]'s lock without restoring the interrupt flag, for panic paths.
///
/// # Safety
/// Same contract as [`IrqMutex::force_unlock`]: only call from a panic handler that
/// halts immediately after and never accesses [`KPRINT_STATE`] again.
pub unsafe fn force_unlock_kprint_state() {
    unsafe { KPRINT_STATE.force_unlock(); }
}

/// RAII handle that holds the [`KPRINT_STATE`] lock for one `kprint!` call, preventing interleaved output.
pub struct KernelWriter {
    lock: IrqMutexGuard<'static, KernelPrintState>
}
impl KernelWriter {
    /// Acquire [`KPRINT_STATE`] and return a [`KernelWriter`] holding the guard.
    pub fn lock() -> Self {
        Self { lock: KPRINT_STATE.lock() }
    }

    /// Write `string` with an optional color to all active output sinks.
    pub fn print_raw(&mut self, string: &str, color: Option<u32>) {
        kprint(&mut self.lock, string, color);
    }

    pub fn print_level_marker(&mut self) {
        write_log_buf(&mut self.lock, LOG_LEVEL_MARKER);
    }
}
impl core::fmt::Write for KernelWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        kprint(&mut self.lock, string, None);
        Ok(())
    }
}

/// Print a formatted message (like `print!`) to all active kernel output sinks.
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

/// Log a line at emergency severity (system is unusable).
#[macro_export]
macro_rules! kemerg {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_level_marker();
            w.print_raw("! ", Some(LogLevelColor::Emergency as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
/// Log a line at alert severity (action must be taken immediately).
#[macro_export]
macro_rules! kalert {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_level_marker();
            w.print_raw("A ", Some(LogLevelColor::Alert as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
/// Log a line at critical severity.
#[macro_export]
macro_rules! kcrit {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_level_marker();
            w.print_raw("C ", Some(LogLevelColor::Critical as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
/// Log a line at error severity.
#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_level_marker();
            w.print_raw("E ", Some(LogLevelColor::Error as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
/// Log a line at warning severity.
#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_level_marker();
            w.print_raw("W ", Some(LogLevelColor::Warn as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
/// Log a line at info severity.
#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => {
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_level_marker();
            w.print_raw("I ", Some(LogLevelColor::Info as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
/// Log a line at debug severity; compiled out unless the `debug-logging` feature is enabled.
#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-logging")]
        {
            use $crate::logging::kprint::{KernelWriter, LogLevelColor};
            let mut w = KernelWriter::lock();
            w.print_level_marker();
            w.print_raw("D ", Some(LogLevelColor::Debug as u32));
            let _ = core::fmt::write(&mut w, format_args!($($arg)*));
            w.print_raw("\n", None);
        }
    };
}
