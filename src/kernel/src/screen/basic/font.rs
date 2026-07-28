// SPDX-License-Identifier: GPL-3.0-only
//! PSF2 bitmap font loading and character rendering.
//!
//! PSF2 (PC Screen Font version 2) is a simple binary format: a fixed-size header
//! followed by a packed array of glyph bitmaps. Each glyph is `bytes_per_glyph`
//! bytes wide, laid out as `height` rows of `ceil(width / 8)` bytes each. Bit 7
//! of each byte corresponds to the leftmost pixel of that byte's group of 8.
//!
//! The font file is embedded at compile time via `include_bytes!`. Glyphs are
//! indexed by Unicode codepoint; characters with a codepoint at or above
//! `glyph_count` are silently skipped.
//!
//! Authors: MarioS271

use crate::screen::basic::framebuffer::BasicFramebuffer;

/// Expected first four bytes of a valid PSF2 file.
static MAGIC_NUMBER: u32 = 0x864ab572;
/// Pixel color written for bits that are 0 in the glyph bitmap (background).
static BACKGROUND_COLOR: u32 = 0x00000000;
/// Pixel color used when the caller passes `None` as the foreground color.
static DEFAULT_FOREGROUND_COLOR: u32 = 0x00FFFFFF;

/// A loaded and parsed PSF2 font, ready for rendering into a [`BasicFramebuffer`].
pub struct Psf2Font {
    /// The raw font file bytes (glyph bitmaps start at `header.header_size`).
    font: &'static [u8],
    header: Psf2Header,
}

impl Psf2Font {
    /// Return the height of one glyph in pixels. Used by `kprint` to calculate
    /// how far to advance `y` after a newline and how many rows to scroll.
    pub fn glyph_height(&self) -> usize {
        self.header.height as usize
    }
}

/// Parsed PSF2 file header. Only the fields needed for rendering are extracted.
struct Psf2Header {
    pub header_size: u32,
    pub glyph_count: u32,
    pub bytes_per_glyph: u32,
    pub height: u32,
    pub width: u32,
}

impl Psf2Font {
    /// Load and parse the built-in PSF2 font embedded in the kernel binary.
    ///
    /// The font file `ter-powerline-v16n.psf` is included at compile time. The
    /// magic number in the file header is verified, and a `Psf2Header` is extracted
    /// from fixed byte offsets as specified by the PSF2 format.
    ///
    /// # Panics
    /// Panics if the embedded file's magic number does not match [`MAGIC_NUMBER`].
    pub fn init() -> Self {
        let font: &[u8] = include_bytes!("../../../resources/ter-powerline-v16n.psf");
        Self {
            font: font,
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

    /// Render one character into the framebuffer at pixel position `(x, y)`.
    ///
    /// Returns the x position immediately after the glyph, or the original `x` if the
    /// character has no glyph or falls outside the framebuffer bounds.
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

impl Psf2Header {
    /// Extract header fields from a raw PSF2 byte slice.
    ///
    /// # Panics
    /// Panics if the magic number at offset 0 does not match [`MAGIC_NUMBER`].
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
