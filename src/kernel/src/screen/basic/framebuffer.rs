//! framebuffer
//! Basic Framebuffer received from Limine, used in early boot for logging
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

// use crate::types::framebuffer::FramebufferData;
use spin::Once;
use limine::framebuffer::Framebuffer;

static BASIC_FRAMEBUFFER: Once<BasicFramebufferData> = Once::new();

pub struct BasicFramebufferData {
    pub fb_pointer: *mut u32,
    pub pixel_stride: u32,
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
            pixel_stride: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
        }
    );
}

pub fn get_framebuffer() -> Option<&'static BasicFramebufferData> {
    BASIC_FRAMEBUFFER.get()
}

pub fn clear_framebuffer(fb: &BasicFramebufferData) {
    for y in 0..fb.height {
        for x in 0..fb.width {
            // Safe because we're iterating inside the given fb bounds and only
            // changing memory there
            unsafe {
                fb.fb_pointer
                    .add(y as usize * fb.pixel_stride as usize + x as usize)
                    .write_volatile(0u32);
            }
        }
    }
}