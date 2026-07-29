// SPDX-License-Identifier: GPL-3.0-only
//! Kernel entry point and early-boot global state.
//!
//! Authors: MarioS271

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
mod state;
mod config;

use spin::Once;
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};
use crate::arch::instructions;
use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Early-boot singleton resources; written once in `kmain`, then read-only.
///
/// Kept separate from [`KState`] so the panic handler still has a reliable source
/// of state if `KState` is unavailable.
struct SimpleKernelState {
    serial: Once<logging::serial::Serial>,
    basic_fb: Once<screen::basic::framebuffer::BasicFramebuffer>,
    basic_fb_psf2_font: Once<screen::basic::font::Psf2Font>,
    pmm: Once<mem::pmm::Pmm>,
    vmm: Once<mem::vmm::Vmm>,
}

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Global early-boot state, readable from anywhere in the kernel once initialized.
pub(crate) static SIMPLE_STATE: SimpleKernelState = SimpleKernelState {
    serial: Once::new(),
    basic_fb: Once::new(),
    basic_fb_psf2_font: Once::new(),
    pmm: Once::new(),
    vmm: Once::new(),
};

/// Kernel entry point called by the Limine bootloader; runs boot init and never returns.
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

    kinfo!("Hello, FerriteOS!");
    kdebug!("Debug logging is active!");

    // Init arch-specific features (GDT, IDT for x86_64, ...)
    arch::init();

    {
        // Init physical + virtual mem manager
        if let Some(memmap_response) = LIMINE_MEMMAP_REQUEST.response() {
            if let Some(hhdm_response) = LIMINE_HHDM_REQUEST.response() {
                SIMPLE_STATE.pmm.call_once(|| mem::pmm::Pmm::init(memmap_response.entries(), hhdm_response.offset) );
                SIMPLE_STATE.vmm.call_once(|| mem::vmm::Vmm::init(SIMPLE_STATE.pmm.get().unwrap(), hhdm_response.offset));
            }
        }
    }

    // Init heap allocator
    mem::heap::init(SIMPLE_STATE.pmm.get().unwrap(), SIMPLE_STATE.vmm.get().unwrap());

    instructions::enable_interrupts();


    kinfo!("Kernel ran successfully!");

    // To halt the kernel on finish (temporary)
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem))
        }
    }
}
