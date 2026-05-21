#![feature(abi_x86_interrupt)]
//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

#![no_std]
#![no_main]

mod types;
mod panic;
mod arch;
mod logging;
mod screen;

use limine::request::FramebufferRequest;

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    // Init logging
    logging::serial::init_com1();

    if let Some(limine_fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response() {
        if let Some(limine_fb) = limine_fb_response.framebuffers().first() {
            screen::basic::framebuffer::init_framebuffer(limine_fb);
            screen::basic::font::init_font_header();
        }
    }

    kprint!("Hello, FerriteOS!\n");


    // Init TSS, GDT, IDT
    arch::tss::init();
    arch::gdt::init();
    arch::idt::init();


    // To halt the kernel on finish (temporary)
    panic::kernel_panic(
        types::panic_codes::PanicCode::ManuallyTriggeredPanic,
        "Kernel executed successfully! :)", 
        true
    );
}