//! screen/basic/framebuffer.rs
//! Basic Framebuffer received from Limine, used in early boot for logging
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use spin::Once;
use limine::framebuffer::Framebuffer;

static BASIC_FRAMEBUFFER: Once<BasicFramebufferData> = Once::new();

pub struct BasicFramebufferData {
    pub fb_pointer: *mut u32,
    pub bytes_per_row: u32,
    pub width: u64,
    pub height: u64,
}

// This is safe because BASIC_FRAMEBUFFER is wrapped in a Once, which makes
// the raw pointer thread-safe
unsafe impl Send for BasicFramebufferData {}
unsafe impl Sync for BasicFramebufferData {}

pub fn init_framebuffer(limine_fb: &Framebuffer) {
    BASIC_FRAMEBUFFER.call_once(
        || BasicFramebufferData{
            fb_pointer: limine_fb.address() as *mut u32,
            bytes_per_row: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
        }
    );
}

pub fn get_framebuffer() -> Option<&'static BasicFramebufferData> {
    BASIC_FRAMEBUFFER.get()
}

impl BasicFramebufferData {
    pub fn clear(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                // Safe because we're iterating inside the given fb bounds and only
                // changing memory there
                unsafe {
                    self.fb_pointer
                        .add(y as usize * self.bytes_per_row as usize + x as usize)
                        .write_volatile(0u32);
                }
            }
        }
    }
}
