// SPDX-License-Identifier: GPL-3.0-only
//! Basic FB KPrint Receiver, receives kprint output to log to the basic framebuffer (if active)
//!
//! Authors: MarioS271

use super::KPrintReceiver;
use crate::kprint::log_entry::{LogEntryHeader, LogLevel};
use crate::kprint::state::KPrintState;
use crate::state::simple_state::SIMPLE_STATE;

pub struct BasicFbReceiver;

impl KPrintReceiver for BasicFbReceiver {
    fn receive(state: &mut KPrintState, header: &LogEntryHeader, text: &str) {
        if !SIMPLE_STATE.is_basic_fb_initialized() || !SIMPLE_STATE.is_basic_fb_psf2_font_initialized() {
            return;
        }

        let log_level_u8 = header.flags_and_level.get(
            LogEntryHeader::LEVEL_START_BIT,
            LogEntryHeader::LEVEL_NUM_BITS
        );
        let log_level = LogLevel::from_u8(log_level_u8);
        if log_level.is_none() { return; }
        let log_level = log_level.unwrap();
        
        let log_level_color = log_level.get_color();
        let log_level_prefix = [log_level.get_letter(), b' '];
        // Safety: log_level.get_letter() can only return valid characters
        let log_level_prefix_str = unsafe {
            core::str::from_utf8_unchecked(&log_level_prefix)
        };

        let cursor_x = &mut state.basic_fb_state.cursor_x;
        let cursor_y = &mut state.basic_fb_state.cursor_y;

        // Safety: was already initialized in stage 1
        let font = unsafe { SIMPLE_STATE.basic_fb_psf2_font() };
        // Safety: was already initialized in stage 1
        let fb = unsafe { SIMPLE_STATE.basic_fb().lock() };

        // Draw Log Level Prefix
        font.draw_string(
            &*fb,
            log_level_prefix_str,
            cursor_x,
            cursor_y,
            Some(log_level_color)
        );
        // Draw actual text
        font.draw_string(
            &*fb,
            text,
            cursor_x,
            cursor_y,
            None
        );

        let last_row = fb.height - font.glyph_height();
        if *cursor_y > last_row {
            *cursor_y = last_row;

            // Safety: src and dst are within framebuffer bounds, counts derived from fb dimensions
            unsafe {
                let src = fb.fb_pointer.add(font.glyph_height() * fb.pixels_per_row);
                let dst = fb.fb_pointer;
                let count = (fb.height - font.glyph_height()) * fb.pixels_per_row;
                core::ptr::copy(src, dst, count);
            }

            // Safety: iteration range is within framebuffer bounds
            unsafe {
                let start = fb.fb_pointer.add((fb.height - font.glyph_height()) * fb.pixels_per_row);
                core::ptr::write_bytes(start, 0u8, fb.pixels_per_row * font.glyph_height());
            }
        }
    }
}
