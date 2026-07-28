//! logging/kprint.rs
//! Logging Macros
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::types::irq_mutex::IrqMutex;

const LOG_BUFFER_SIZE: usize = u16::MAX as usize;

struct KernelPrintState {
    log_buf: [u8; LOG_BUFFER_SIZE],
    log_head: usize,
    log_tail: usize,
    cursor_x: usize,
    cursor_y: usize,
}

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

pub struct KernelWriter;
impl core::fmt::Write for KernelWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        kprint(string);
        Ok(())
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ({
        let _ = core::fmt::write(
            &mut $crate::logging::kprint::KernelWriter,
            format_args!($($arg)*)
        );
    });
}
