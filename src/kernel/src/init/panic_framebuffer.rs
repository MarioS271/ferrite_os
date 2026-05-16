//! panic_framebuffer.rs
//! Panic Framebuffer, used on panic
//!
//! The panic framebuffer is the same as the basic framebuffer, just accessed differently to
//! prevent deadlocks when panicking while the basic framebuffer mutex is locked
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use spin::Once;
use limine::framebuffer::Framebuffer;

static PANIC_FRAMEBUFFER: Once<PanicFramebufferData> = Once::new();

struct PanicFramebufferData {
    fb_pointer: *mut u32,
    pixel_stride: u32,
    width: u64,
    height: u64,
}

// This is safe because PanicFramebufferData is wrapped in a Once().
// The data will only be written BEFORE parallelism exists.
// Afterward, the data will become read-only, making Send and Sync thread-safe.
unsafe impl Send for PanicFramebufferData {}
unsafe impl Sync for PanicFramebufferData {}

pub fn init_framebuffer(limine_fb: &Framebuffer) {
    PANIC_FRAMEBUFFER.call_once(
        || PanicFramebufferData{
            fb_pointer: limine_fb.address() as *mut u32,
            pixel_stride: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
        }
    );
}

pub fn get_framebuffer() -> Option<&'static PanicFramebufferData> {
    PANIC_FRAMEBUFFER.get()
}