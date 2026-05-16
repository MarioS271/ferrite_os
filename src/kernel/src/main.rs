//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

#![no_std]
#![no_main]

mod panic;
mod init;

use limine::request::FramebufferRequest;

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    if let Some(limine_fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response() {
        if let Some(limine_fb) = limine_fb_response.framebuffers().first() {
            init::basic_framebuffer::init_framebuffer(limine_fb);
            init::panic_framebuffer::init_framebuffer(limine_fb);
        }
    }

    panic!();

    // if let Some(fb_response) = FRAMEBUFFER.response() {
    //     if let Some(fb) = fb_response.framebuffers().first() {
    //         let ptr = fb.address() as *mut u32;
    //         let pixels = (fb.pitch / 4) as usize * fb.height as usize;
    //         for i in 0..pixels {
    //             unsafe { ptr.add(i).write_volatile(0x00_AA_FF_00) };
    //         }
    //     }
    // }
}