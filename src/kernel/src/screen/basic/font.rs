//! font.rs
//! Kernel font(s) and character rendering
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::panic::kernel_panic;
use crate::screen::basic::framebuffer::BasicFramebufferData;
use crate::types::panic_codes::PanicCode;
use core::convert::TryInto;
use spin::Once;

static TERMINUS_POWERLINE_16: &[u8] = include_bytes!("../../../resources/ter-powerline-v16n.psf");
pub static TERMINUS_POWERLINE_16_HEADER: Once<Psf2Header> = Once::new();

static MAGIC_NUMBER: u32 = 0x864ab572;
static BACKGROUND_COLOR: u32 = 0x00000000;
static DEFAULT_FOREGROUND_COLOR: u32 = 0x00FFFFFF;

pub struct Psf2Header {
    pub header_size: u32,
    pub glyph_count: u32,
    pub bytes_per_glyph: u32,
    pub height: u32,
    pub width: u32,
}

fn parse_psf2_header(psf2_font: &[u8]) -> Psf2Header {
    let read_u32 = |offset: usize| -> u32 {
        u32::from_le_bytes(psf2_font[offset..(offset + 4)].try_into().unwrap())
    };

    if read_u32(0) != MAGIC_NUMBER {
        kernel_panic(
            PanicCode::InvalidPsf2MagicNumber,
            "Invalid PSF2 Magic Number",
            false
        );
    }

    let header: Psf2Header = Psf2Header{
        header_size: read_u32(8),
        glyph_count: read_u32(16),
        bytes_per_glyph: read_u32(20),
        height: read_u32(24),
        width: read_u32(28),
    };

    header
}

fn parse_char(c: char, header: &Psf2Header) -> Option<&'static [u8]> {
    let char_index = c as usize;

    if char_index >= header.glyph_count as usize {
        return None;
    }

    let start = header.header_size as usize + char_index * header.bytes_per_glyph as usize;

    Some(&TERMINUS_POWERLINE_16[start..(start + header.bytes_per_glyph as usize)])
}

pub fn init_font_header() {
    TERMINUS_POWERLINE_16_HEADER.call_once(|| parse_psf2_header(TERMINUS_POWERLINE_16));
}

/// If None as the font color is passed, fallback will be 0x00FFFFFF (white)
/// Returns the x position after the drawn character so callers can chain writes
/// Returns the original x position if the drawing fails
pub fn draw_char(fb: &BasicFramebufferData, c: char, x: usize, y: usize, font_color: Option<u32>) -> usize {
    let Some(header) = TERMINUS_POWERLINE_16_HEADER.get() else { return x; };
    let Some(glyph) = parse_char(c, &header) else { return x; };

    if x + header.width as usize > fb.width as usize
        || y + header.height as usize > fb.height as usize
    {
        return x;
    }

    let row_stride = (header.width + 7) / 8;

    for row in 0..header.height as usize {
        for col in 0..header.width as usize {
            let byte = glyph[row * row_stride as usize + col / 8];
            let bit = (byte >> (7 - (col % 8))) & 1;

            let color = if bit == 1 {
                font_color.unwrap_or(DEFAULT_FOREGROUND_COLOR)
            } else {
                BACKGROUND_COLOR
            };
            let pixel_pos = (y + row) * fb.pixel_stride as usize + (x + col);

            // This is safe because the above if statement would've returned if we were
            // in invalid memory
            unsafe {
                fb.fb_pointer.add(pixel_pos).write_volatile(color);
            }
        }
    }

    x + header.width as usize
}
