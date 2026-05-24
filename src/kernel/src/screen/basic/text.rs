//! screen/basic/text.rs
//! String drawing and printing to the basic framebuffer
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use spin::Mutex;
use crate::screen::basic::framebuffer::BasicFramebufferData;

/// Directly uses the given x and y as cursor (changes them)
pub fn draw_string(fb: &BasicFramebufferData, string: &str, x: &mut usize, y: &mut usize, font_color: Option<u32>) {
    let Some(font_header) = super::font::TERMINUS_POWERLINE_16_HEADER.get() else { return; };

    for c in string.chars() {
        if c == '\n' {
            *x = 0;
            *y += font_header.height as usize;
        } else {
            *x = super::font::draw_char(fb, c, *x, *y, font_color);
        }
    }
}

/// Wrapper of draw_string
/// Returns the x and y position after drawing as a tuple instead of directly modifying
/// the given x and y reference
pub fn draw_string_no_mut(fb: &BasicFramebufferData, string: &str, x: usize, y: usize, font_color: Option<u32>) -> (usize, usize) {
    let mut cursor_x = x;
    let mut cursor_y = y;

    draw_string(fb, string, &mut cursor_x, &mut cursor_y, font_color);

    (cursor_x, cursor_y)
}

struct VgaPrintState {
    x: usize,
    y: usize,
}
static VGA_PRINT_STATE: Mutex<VgaPrintState> = Mutex::new(VgaPrintState{x: 0, y: 0});

pub fn print_to_basic_fb(string: &str) {
    use super::font::TERMINUS_POWERLINE_16_HEADER;

    let font_header = TERMINUS_POWERLINE_16_HEADER.get().unwrap();
    let mut state_lock = VGA_PRINT_STATE.lock();

    if let Some(fb) = super::framebuffer::get_framebuffer() {
        let last_row = fb.height as usize - font_header.height as usize;

        let mut x = state_lock.x;
        let mut y = state_lock.y;

        draw_string(fb, string, &mut x, &mut y, None);

        state_lock.x = x;
        state_lock.y = y;

        if state_lock.y > last_row {
            state_lock.y = last_row;

            // Safe because the source and destination are both within the framebuffer's
            // memory range. Source and destination are computed from the framebuffer bounds,
            // meaning they will not go out of bounds.
            unsafe {
                let src = fb.fb_pointer.add(font_header.height as usize * fb.pixel_stride as usize);
                let dst = fb.fb_pointer;
                let count = (fb.height as usize - font_header.height as usize) * fb.pixel_stride as usize;

                core::ptr::copy(src, dst, count);
            }

            // This is safe because we are iterating inside the framebuffer memory again,
            // with the bounds of the iteration computed from framebuffer bounds.
            unsafe {
                let start = (fb.height as usize - font_header.height as usize) * fb.pixel_stride as usize;
                let end = fb.height as usize * fb.pixel_stride as usize;

                for i in start..end {
                    fb.fb_pointer.add(i).write_volatile(0u32);
                }
            }
        }
    }
}