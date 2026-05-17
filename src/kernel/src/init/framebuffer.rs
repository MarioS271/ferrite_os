//! framebuffer
//! Basic Framebuffer received from Limine, used in early boot for logging
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::data_structures::framebuffer::FramebufferData;
use spin::Mutex;
use limine::framebuffer::Framebuffer;

static BASIC_FRAMEBUFFER: Mutex<Option<FramebufferData>> = Mutex::new(None);

pub fn init_framebuffer(limine_fb: &Framebuffer) {
    let mut fb_lock = BASIC_FRAMEBUFFER.lock();

    *fb_lock = Some(
        FramebufferData{
            fb_pointer: limine_fb.address() as *mut u32,
            pixel_stride: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
            x: 0,
            y: 0,
        }
    );
}

pub fn get_framebuffer() -> &'static Mutex<Option<FramebufferData>> {
    &BASIC_FRAMEBUFFER
}