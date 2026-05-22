//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod types;
mod panic;
mod arch;
mod logging;
mod screen;
mod mem;

use limine::request::{FramebufferRequest, MemmapRequest};

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    // Init logging
    logging::serial::init_com1();

    if let Some(fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response() {
        if let Some(fb) = fb_response.framebuffers().first() {
            screen::basic::framebuffer::init_framebuffer(fb);
            screen::basic::font::init_font_header();
        }
    }

    kprint!("Hello, FerriteOS!\n");


    // Init TSS, GDT, IDT
    arch::tss::init();
    arch::gdt::init();
    arch::idt::init();


    // Init PMM
    if let Some(memmap_response) = LIMINE_MEMMAP_REQUEST.response() {
        mem::pmm::init(memmap_response.entries());
    }


    // To halt the kernel on finish (temporary)
    panic::kernel_panic(
        types::panic_codes::PanicCode::ManuallyTriggeredPanic,
        "Kernel executed successfully! :)", 
        true
    );
}