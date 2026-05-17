//! framebuffer
//! Panic Framebuffer, used on panic
//!
//! The panic panic is the same as the basic panic, just accessed differently to
//! prevent deadlocks when panicking while the basic panic mutex is locked
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::data_structures::framebuffer::FramebufferData;
use spin::Once;
use limine::framebuffer::Framebuffer;

static PANIC_FRAMEBUFFER: Once<FramebufferData> = Once::new();

pub fn init_framebuffer(limine_fb: &Framebuffer) {
    PANIC_FRAMEBUFFER.call_once(
        || FramebufferData{
            fb_pointer: limine_fb.address() as *mut u32,
            pixel_stride: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
            x: 0,
            y: 0,
        }
    );
}

pub fn get_framebuffer() -> Option<&'static FramebufferData> {
    PANIC_FRAMEBUFFER.get()
}