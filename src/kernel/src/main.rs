//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

#![no_std]
#![no_main]

mod panic;

use limine::request::FramebufferRequest;

static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    if let Some(fb_response) = FRAMEBUFFER.response() {
        if let Some(fb) = fb_response.framebuffers().first() {
            let ptr = fb.address() as *mut u32;
            let pixels = (fb.pitch / 4) as usize * fb.height as usize;
            for i in 0..pixels {
                unsafe { ptr.add(i).write_volatile(0x00_AA_FF_00) };
            }
        }
    }
    loop {}
}