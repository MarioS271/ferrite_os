//! text.rs
//! String drawing using draw_char from font.rs
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

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