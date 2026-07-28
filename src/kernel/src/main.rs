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

use spin::Once;
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};
use crate::arch::instructions;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

struct SimpleKernelState {
    serial: Once<logging::serial::Serial>,
    basic_fb: Once<screen::basic::framebuffer::BasicFramebuffer>,
    basic_fb_psf2_font: Once<screen::basic::font::Psf2Font>,
}

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

pub static SIMPLE_STATE: SimpleKernelState = SimpleKernelState {
    serial: Once::new(),
    basic_fb: Once::new(),
    basic_fb_psf2_font: Once::new(),
};

#[no_mangle]
extern "C" fn kmain() -> ! {
    {
        use logging::serial::Serial;
        use logging::_serial::{SerialPort, _Serial};

        SIMPLE_STATE.serial.call_once(|| -> Serial {
            Serial::new(SerialPort::Serial1)
        });
        let res = SIMPLE_STATE.serial.get().unwrap().init();
        if !res.is_ok() {
            kernel_panic(
                PanicCode::InitFailure,
                "Failed to initialize serial for kernel logging"
            );
        }
    }

    {
        use crate::screen::basic::framebuffer::BasicFramebuffer;
        use crate::screen::basic::font::Psf2Font;

        if let Some(fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response() {
            if let Some(fb) = fb_response.framebuffers().first() {
                SIMPLE_STATE.basic_fb.call_once(|| -> BasicFramebuffer {
                    BasicFramebuffer::new(fb)
                });
                SIMPLE_STATE.basic_fb_psf2_font.call_once(|| -> Psf2Font {
                    Psf2Font::init()
                });
            }
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

    instructions::enable_interrupts();


    kprint!("Kernel ran successfully!\n");

    // To halt the kernel on finish (temporary)
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem))
        }
    }
}
