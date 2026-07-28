// SPDX-License-Identifier: GPL-3.0-only
//! Kernel entry point and early-boot global state.
//!
//! `kmain` is the first Rust function called by the Limine bootloader. It runs the full
//! initialization sequence in order: serial → framebuffer → arch tables → PMM → VMM → heap →
//! interrupts. Order matters because later steps depend on earlier ones being complete.
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

/// Early-boot singleton resources, written once in `kmain` then read-only.
/// `SIMPLE_STATE` is the only instance.
struct SimpleKernelState {
    /// The COM1 serial port, used for early kernel logging before the heap is available.
    serial: Once<logging::serial::Serial>,
    /// The Limine-provided linear framebuffer, used for on-screen text output.
    basic_fb: Once<screen::basic::framebuffer::BasicFramebuffer>,
    /// The PSF2 bitmap font used to render characters into `basic_fb`.
    basic_fb_psf2_font: Once<screen::basic::font::Psf2Font>,
}

/// Limine bootloader request for the linear framebuffer.
///
/// Limine fills this before calling `kmain`. If the response is `None`, no
/// framebuffer is available and output falls back to serial only.
static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Limine bootloader request for the physical memory map.
///
/// The response contains a list of [`limine::memmap::Entry`] regions describing
/// which physical address ranges are usable, reserved, etc. The PMM consumes this.
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

/// Limine bootloader request for the Higher-Half Direct Map (HHDM) offset.
///
/// Limine maps all physical memory at `hhdm_offset + physical_address`. Every
/// subsystem that needs to convert between physical and virtual addresses
/// (PMM, VMM, heap) reads this offset.
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Global early-boot state, accessible from anywhere in the kernel.
///
/// Fields are initialized in `kmain` via `call_once` and then read-only. Callers
/// should check `is_completed()` before calling `get()` — for example, the panic
/// handler does this to avoid crashing while printing a crash message.
pub(crate) static SIMPLE_STATE: SimpleKernelState = SimpleKernelState {
    serial: Once::new(),
    basic_fb: Once::new(),
    basic_fb_psf2_font: Once::new(),
};

/// Kernel entry point called by the Limine bootloader.
///
/// Runs the complete boot initialization sequence. Never returns; the final
/// loop issues `hlt` to idle the CPU between timer interrupts.
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
