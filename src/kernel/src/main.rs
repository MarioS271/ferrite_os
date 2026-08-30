// SPDX-License-Identifier: GPL-3.0-only
//! Kernel entry point and early-boot global state.
//!
//! Authors: MarioS271

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod init;
mod logging;
mod mem;
mod screen;
mod state;
mod types;

mod panic;

use crate::arch::instructions;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::state::simple_state::SIMPLE_STATE;
use crate::types::panic_codes::PanicCode;
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

unsafe extern "C" {
    /// Symbol which is located at the start of the kernel
    pub static __kernel_start: u8;
    /// Symbol which is located at the end of the kernel
    pub static __kernel_end: u8;

    /// Symbol which is located one byte after the end of the `.text` section
    pub static __kernel_text_end: u8;
    /// Symbol which is located one byte after the end of the `.rodata` section
    pub static __kernel_rodata_end: u8;
}

/// Kernel entry point
#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    serial_init();
    basic_fb_init();
    early_kstate_populate();

    kinfo!("Hello, Ferrite!");
    kdebug!("Debug logging is active!");

    arch::init();
    init::mem::mm_init();
    instructions::enable_interrupts();

    kinfo!("Kernel ran successfully!");

    // To halt the kernel on finish (temporary)
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem))
        }
    }
}

/// Initializes the kernel serial logger
fn serial_init() {
    use crate::logging::serial::{Serial, SerialPort, _Serial};

    SIMPLE_STATE.init_serial(
        Serial::new(SerialPort::Serial1)
    );
}

/// Initializes the Basic Framebuffer
fn basic_fb_init() {
    use crate::screen::basic::framebuffer::BasicFramebuffer;
    use crate::screen::basic::font::Psf2Font;

    if let Some(fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response()
        && let Some(fb) = fb_response.framebuffers().first()
    {
        SIMPLE_STATE.init_basic_fb(BasicFramebuffer::new(fb));
        SIMPLE_STATE.init_basic_fb_psf2_font(Psf2Font::init());
    } else {
        kernel_panic(
            PanicCode::InitFailure,
            "Limine did not provide a valid framebuffer"
        );
    }
}

/// Populates KSTATE with early available info such as the hhdm offset
fn early_kstate_populate() {
    if let Some(hhdm_response) = LIMINE_HHDM_REQUEST.response() {
        KSTATE.mm.set_hhdm_offset(hhdm_response.offset);
    } else {
        kernel_panic(
            PanicCode::InitFailure,
            "Limine did not provide a hddm offset"
        );
    }
}
