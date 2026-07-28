//! screen/basic/font.rs
//! Kernel font(s) and character rendering
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::screen::basic::framebuffer::BasicFramebuffer;

static MAGIC_NUMBER: u32 = 0x864ab572;
static BACKGROUND_COLOR: u32 = 0x00000000;
static DEFAULT_FOREGROUND_COLOR: u32 = 0x00FFFFFF;

pub struct Psf2Font {
    font: &'static [u8],
    header: Psf2Header,
}

impl Psf2Font {
    pub fn glyph_height(&self) -> usize {
        self.header.height as usize
    }
}

struct Psf2Header {
    pub header_size: u32,
    pub glyph_count: u32,
    pub bytes_per_glyph: u32,
    pub height: u32,
    pub width: u32,
}

impl Psf2Font {
    pub fn init() -> Self {
        let font: &[u8] = include_bytes!("../../../resources/ter-powerline-v16n.psf");
        Self {
            font: font,
            header: Psf2Header::parse(font),
        }
    }

    fn parse_char(&self, c: char) -> Option<&'static [u8]> {
        let char_index = c as usize;

        if char_index >= self.header.glyph_count as usize {
            return None;
        }

        let start = self.header.header_size as usize + char_index * self.header.bytes_per_glyph as usize;

        Some(&self.font[start..(start + self.header.bytes_per_glyph as usize)])
    }

    /// If None as the font color is passed, fallback will be 0x00FFFFFF (white)
    /// Returns the x position after the drawn character so callers can chain writes
    /// Returns the original x position if the drawing fails
    pub fn draw_char(&self, fb: &BasicFramebuffer, c: char, x: usize, y: usize, font_color: Option<u32>) -> usize {
        let Some(glyph) = self.parse_char(c) else { return x; };

        if x + self.header.width as usize > fb.width as usize
            || y + self.header.height as usize > fb.height as usize
        {
            return x;
        }

        let row_stride = (self.header.width + 7) / 8;

        for row in 0..self.header.height as usize {
            for col in 0..self.header.width as usize {
                let byte = glyph[row * row_stride as usize + col / 8];
                let bit = (byte >> (7 - (col % 8))) & 1;

                let color = if bit == 1 {
                    font_color.unwrap_or(DEFAULT_FOREGROUND_COLOR)
                } else {
                    BACKGROUND_COLOR
                };
                let pixel_pos = (y + row) * fb.bytes_per_row as usize + (x + col);

                // This is safe because the above if statement would've returned if we were
                // in invalid memory
                unsafe {
                    fb.fb_pointer.add(pixel_pos).write_volatile(color);
                }
            }
        }

        x + self.header.width as usize
    }

    /// Directly uses the given x and y as cursor (changes them)
    pub fn draw_string(&self, fb: &BasicFramebuffer, string: &str, x: &mut usize, y: &mut usize, font_color: Option<u32>) {
        for c in string.chars() {
            if c == '\n' {
                *x = 0;
                *y += self.header.height as usize;
            } else {
                *x = self.draw_char(fb, c, *x, *y, font_color);
            }
        }
    }
}

impl Psf2Header {
    fn parse(psf2_font: &[u8]) -> Self {
        let read_u32 = |offset: usize| -> u32 {
            u32::from_le_bytes(psf2_font[offset..(offset + 4)].try_into().unwrap())
        };

        if read_u32(0) != MAGIC_NUMBER {
            use crate::panic::kernel_panic;
            use crate::types::panic_codes::PanicCode;

            kernel_panic(
                PanicCode::InvalidPsf2MagicNumber,
                "Invalid PSF2 Magic Number",
            );
        }

        Self {
            header_size: read_u32(8),
            glyph_count: read_u32(16),
            bytes_per_glyph: read_u32(20),
            height: read_u32(24),
            width: read_u32(28),
        }
    }
}
