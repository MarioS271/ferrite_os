//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

#![no_std]
#![no_main]

mod panic;
mod screen;
mod types;
mod logging;

use limine::request::FramebufferRequest;

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    logging::serial::init_com1();
    logging::serial::write_string_to_com1("Hello, FerriteOS!\n");

    if let Some(limine_fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response() {
        if let Some(limine_fb) = limine_fb_response.framebuffers().first() {
            screen::basic::framebuffer::init_framebuffer(limine_fb);
        }
    }

    screen::basic::font::init_font_header();

    let number = 3;

    kprint!("test hello world\n");
    kprint!("i have a number, ");
    kprint!("its an i32 which says: {number}\n");

    for i in 1..51 {
        kprint!("we're on iteration: {i}\n");
    }

    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nostack, nomem));
        }
    }

    panic::kernel_panic(
        types::panic_codes::PanicCode::ManuallyTriggeredPanic,
        "Kernel executed successfully! :)",
        true
    );
}