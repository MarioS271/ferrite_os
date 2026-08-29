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
use crate::types::panic_codes::PanicCode;
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};
use spin::Once;

/// Data structure for keeping the serial logger and basic fb resources for early boot and panic
struct SimpleKernelState {
    serial: Once<logging::serial::Serial>,
    basic_fb: Once<screen::basic::framebuffer::BasicFramebuffer>,
    basic_fb_psf2_font: Once<screen::basic::font::Psf2Font>,
}

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Data structure for keeping the serial logger and basic fb resources for early boot and panic
pub(crate) static SIMPLE_STATE: SimpleKernelState = SimpleKernelState {
    serial: Once::new(),
    basic_fb: Once::new(),
    basic_fb_psf2_font: Once::new(),
};

/// Symbol which shows the last address of the kernel
unsafe extern "C" {
    pub static __kernel_start: u8;
    pub static __kernel_end: u8;
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

/// Initializes COM1
fn serial_init() {
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

/// Initializes the Basic Framebuffer
fn basic_fb_init() {
    use crate::screen::basic::framebuffer::BasicFramebuffer;
    use crate::screen::basic::font::Psf2Font;

    if let Some(fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response()
        && let Some(fb) = fb_response.framebuffers().first()
    {
        SIMPLE_STATE.basic_fb.call_once(|| BasicFramebuffer::new(fb));
        SIMPLE_STATE.basic_fb_psf2_font.call_once(|| Psf2Font::init());
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
