//! basic_framebuffer.rs
//! Basic Framebuffer received from Limine, used in early boot for logging
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use spin::Mutex;
use limine::framebuffer::Framebuffer;

static BASIC_FRAMEBUFFER: Mutex<Option<BasicFramebufferData>> = Mutex::new(None);

struct BasicFramebufferData {
    fb_pointer: *mut u32,
    pixel_stride: u32,
    width: u64,
    height: u64,
    x: usize,
    y: usize,
}

// This is safe because BasicFramebufferData is wrapped in a Mutex().
// The mutex ensures BasicFramebufferData can only be accessed by one caller at a time.
unsafe impl Send for BasicFramebufferData {}
unsafe impl Sync for BasicFramebufferData {}

pub fn init_framebuffer(limine_fb: &Framebuffer) {
    let mut fb_lock = BASIC_FRAMEBUFFER.lock();

    *fb_lock = Some(
        BasicFramebufferData{
            fb_pointer: limine_fb.address() as *mut u32,
            pixel_stride: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
            x: 0,
            y: 0,
        }
    );
}

pub fn get_framebuffer() -> &'static Mutex<Option<BasicFramebufferData>> {
    &BASIC_FRAMEBUFFER
}