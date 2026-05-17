//! font.rs
//! Kernel font(s)
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::panic::framebuffer;
use core::convert::TryInto;
use std::panic;

static TERMINUS_POWERLINE_16: &[u8] = include_bytes!("../../resources/ter-powerline-v16n.psf");
static MAGIC_NUMBER: u32 = 0x864ab572;

struct Psf2Header {
    header_size: u32,
    glyph_count: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
}

fn parse_psf2_header() -> Option<Psf2Header> {
    let read_u32 = |offset: usize| -> u32 {
        u32::from_le_bytes(TERMINUS_POWERLINE_16[offset..(offset + 4)].try_into().unwrap())
    };

    if read_u32(0) != MAGIC_NUMBER {
        return None;
    }

    let header: Psf2Header = Psf2Header{
        header_size: read_u32(8),
        glyph_count: read_u32(16),
        bytes_per_glyph: read_u32(20),
        height: read_u32(24),
        width: read_u32(28),
    };

    Some(header)
}

fn parse_char(c: char, header: &Psf2Header) -> Option<&'static [u8]> {
    let char_index = c as usize;

    if char_index >= header.glyph_count as usize {
        return None;
    }

    let start = header.header_size as usize + char_index * header.bytes_per_glyph as usize;

    Some(&TERMINUS_POWERLINE_16[start..(start + header.bytes_per_glyph as usize)])
}

pub fn draw_char(c: char) {
    let basic_fb = framebuffer::get_framebuffer();

    let Some(header) = parse_psf2_header() else { return; };
    let Some(glyph) = parse_char(c, &header) else { return; };

    let mut fb_lock = basic_fb.lock();
    let Some(fb) = fb_lock.as_mut() else { return; };

    if fb.x + header.width as usize > fb.width as usize
        || fb.y + header.height as usize > fb.height as usize
    {
        return;
    }

    let row_stride = (header.width + 7) / 8;

    for row in 0..header.height as usize {
        for col in 0..header.width as usize {
            let byte = glyph[row * row_stride as usize + col / 8];
            let bit = (byte >> (7 - (col % 8))) & 1;

            let color = if bit == 1 { 0x00FFFFFF } else { 0x00000000 };
            let pixel_pos = (fb.y + row) * fb.pixel_stride as usize + (fb.x + col);

            // As long as we're writing inside panic address space, this is safe
            // It is guaranteed that we're in this address space because of the if block
            // that checks if the written char will exceed the framebuffer bounds and
            // return if so
            unsafe {
                fb.fb_pointer.add(pixel_pos).write_volatile(color);
            }
        }
    }

    fb.x += header.width as usize;
}