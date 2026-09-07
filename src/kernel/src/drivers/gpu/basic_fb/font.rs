// SPDX-License-Identifier: GPL-3.0-only
//! PSF2 bitmap font loading and character rendering.
//!
//! Authors: MarioS271

use super::framebuffer::BasicFramebuffer;

/// A loaded and parsed PSF2 font, ready for rendering into a [`BasicFramebuffer`]
pub struct Psf2Font {
    font: &'static [u8],
    header: Psf2Header,
}
impl Psf2Font {
    const DEFAULT_BACKGROUND_COLOR: u32 = 0x00000000;
    const DEFAULT_FOREGROUND_COLOR: u32 = 0x00FFFFFF;

    /// Load and parse the built-in PSF2 font embedded in the kernel binary.
    ///
    /// # Panics
    /// Panics if the embedded file's magic number does not match [`MAGIC_NUMBER`].
    pub fn init() -> Self {
        let font: &[u8] = include_bytes!("../../../../resources/ter-powerline-v16n.psf");
        Self {
            font,
            header: Psf2Header::parse(font),
        }
    }

    /// Return the raw glyph bytes for character `c`, or `None` if `c` has no glyph.
    fn parse_char(&self, c: char) -> Option<&'static [u8]> {
        let char_index = c as usize;

        if char_index >= self.header.glyph_count as usize {
            return None;
        }

        let start = self.header.header_size as usize + char_index * self.header.bytes_per_glyph as usize;

        Some(&self.font[start..(start + self.header.bytes_per_glyph as usize)])
    }

    /// Return the height of one glyph in pixels.
    pub fn glyph_height(&self) -> usize {
        self.header.height as usize
    }

    /// Render one character into the framebuffer at pixel position `(x, y)`.
    ///
    /// Returns the x position immediately after the glyph, or the original `x` if the
    /// character has no glyph or falls outside the framebuffer bounds.
    pub fn draw_char(&self, fb: &BasicFramebuffer, c: char, x: usize, y: usize, font_color: Option<u32>) -> usize {
        let Some(glyph) = self.parse_char(c) else { return x; };

        if x + self.header.width as usize > fb.width
            || y + self.header.height as usize > fb.height
        {
            return x;
        }

        let row_stride = (self.header.width + 7) / 8;

        for row in 0..self.header.height as usize {
            for col in 0..self.header.width as usize {
                let byte = glyph[row * row_stride as usize + col / 8];
                let bit = (byte >> (7 - (col % 8))) & 1;

                let color = if bit == 1 {
                    font_color.unwrap_or(Self::DEFAULT_FOREGROUND_COLOR)
                } else {
                    Self::DEFAULT_BACKGROUND_COLOR
                };
                let pixel_pos = (y + row) * fb.pixels_per_row + (x + col);

                // Safety: the if statement before the double loop would've returned if we
                // were in invalid memory
                unsafe {
                    fb.fb_pointer.add(pixel_pos).write_volatile(color);
                }
            }
        }

        x + self.header.width as usize
    }

    /// Render a string into the framebuffer, advancing `*x`/`*y` after each character.
    /// `'\n'` wraps to the next line.
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

/// Parsed PSF2 file header
struct Psf2Header {
    pub header_size: u32,
    pub glyph_count: u32,
    pub bytes_per_glyph: u32,
    pub width: u32,
    pub height: u32
}
impl Psf2Header {
    const MAGIC_NUMBER: u32 = 0x864ab572;

    /// Extract header fields from a raw PSF2 byte slice.
    ///
    /// # Panics
    /// Panics if the magic number at offset 0 does not match [`MAGIC_NUMBER`].
    fn parse(psf2_font: &[u8]) -> Self {
        use crate::lib::panic::kernel_panic;
        use crate::lib::panic_codes::PanicCode;

        let read_u32 = |offset: usize| -> u32 {
            u32::from_le_bytes(psf2_font[offset..(offset + 4)].try_into().unwrap())
        };

        if psf2_font.len() < 32 {
            kernel_panic(
                PanicCode::MalformedPsf2Font,
                "PSF2 font is too short"
            );
        }

        if read_u32(0) != Self::MAGIC_NUMBER {
            kernel_panic(
                PanicCode::MalformedPsf2Font,
                "Invalid PSF2 Magic Number",
            );
        }

        let header_size = read_u32(8);
        let glyph_count = read_u32(16);
        let bytes_per_glyph = read_u32(20);
        let width = read_u32(28);
        let height = read_u32(24);

        let end_of_glyphs = (glyph_count as usize)
            .checked_mul(bytes_per_glyph as usize)
            .and_then(|glyph_bytes| glyph_bytes.checked_add(header_size as usize));

        match end_of_glyphs {
            Some(end) if end <= psf2_font.len() => {}
            _ => {
                kernel_panic(
                    PanicCode::MalformedPsf2Font,
                    "PSF2 header declares more glyph data than the font blob contains",
                );
            }
        }

        Self {
            header_size,
            glyph_count,
            bytes_per_glyph,
            width,
            height
        }
    }
}
