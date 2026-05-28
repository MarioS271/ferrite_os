//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod types;
mod panic;
mod arch;
mod logging;
mod screen;
mod mem;

use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

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
    

    // Init arch-specific features (GDT, IDT for x86_64, ...)
    arch::init();

    // Init physical mem manager
    if let Some(memmap_response) = LIMINE_MEMMAP_REQUEST.response() {
        if let Some(hhdm_response) = LIMINE_HHDM_REQUEST.response() {
            mem::pmm::init(memmap_response.entries(), hhdm_response.offset);
        }
    }


    // To halt the kernel on finish (temporary)
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem))
        }
    }
}