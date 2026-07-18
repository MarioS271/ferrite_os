//! main.rs
//! Main kernel entrypoint
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod types;
mod panic;
mod arch;
mod logging;
mod screen;
mod mem;

use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};
use alloc::vec::Vec;
use alloc::boxed::Box;

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    // Init logging
    logging::serial::init_serial();

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
            mem::vmm::init(hhdm_response.offset);
        }
    }

    // Init heap allocator
    mem::heap::init();

    // temp test code for heap alloc
    unsafe {
        // basic allocation
        let mut vec = Vec::new();
        vec.push(0xbeef_u64);
        assert_eq!(vec[0], 0xbeef);

        // multiple live allocs
        let box1 = Box::new(0x0001_u64);
        let box2 = Box::new(0x0002_u64);
        let box3 = Box::new(0x0003_u64);
        assert_eq!(*box1, 0x0001);
        assert_eq!(*box2, 0x0002);
        assert_eq!(*box3, 0x0003);

        // vec growth
        let mut v: Vec<u64> = Vec::new();
        for i in 0..64 {
            v.push(i);
        }
        assert_eq!(v.len(), 64);
        assert_eq!(v[63], 63);

        // dealloc then realloc: drop box2, allocate again, heap must not corrupt
        drop(box2);
        let box4 = Box::new(0xdead_u64);
        assert_eq!(*box4, 0xdead);

        // nested: Vec<Box<u64>>
        let mut nested: Vec<Box<u64>> = Vec::new();
        for i in 0..8 {
            nested.push(Box::new(i * 0x10));
        }
        for i in 0..8_u64 {
            assert_eq!(*nested[i as usize], i * 0x10);
        }
    }


    kprint!("Kernel ran successfully!");

    // To halt the kernel on finish (temporary)
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem))
        }
    }
}
