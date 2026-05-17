//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

#![no_std]
#![no_main]

mod framebuffer;
mod panic;
mod init;
mod data_structures;

use limine::request::FramebufferRequest;

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    if let Some(limine_fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response() {
        if let Some(limine_fb) = limine_fb_response.framebuffers().first() {
            init::framebuffer::init_framebuffer(limine_fb);
            panic::framebuffer::init_framebuffer(limine_fb);
        }
    }

    panic!();
}