// SPDX-License-Identifier: GPL-3.0-only
//! Linear framebuffer wrapper for visual output for early boot and panic.
//!
//! Authors: MarioS271

use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::boot_info::FramebufferInfo;
// TODO: do RGB/BGR check

/// A thin wrapper around the bootloader-provided framebuffer
pub struct BasicFramebuffer {
    pub fb_pointer: *mut u32,
    pub pixels_per_row: usize,
    pub width: usize,
    pub height: usize,
}

/// Safety: `BasicFramebuffer::fb_pointer` points to the valid limine-allocated fb which has a static lifetime
unsafe impl Send for BasicFramebuffer {}

impl BasicFramebuffer {
    /// Constructor; returns a `BasicFramebuffer` constructed from [`FramebufferInfo`]
    pub fn new(fb: &FramebufferInfo) -> Self {
        if fb.bpp != 32 {
            kernel_panic(
                PanicCode::InitFailure,
                "The bootloader provided a bpp value that is not 32, which the kernel doesn't support"
            )
        }
        Self {
            fb_pointer: fb.addr,
            pixels_per_row: fb.pixels_per_row as usize,
            width: fb.width as usize,
            height: fb.height as usize
        }
    }

    /// Write zero (black) to every pixel in the framebuffer
    pub fn clear(&self) {
        // Safety: fb_pointer is valid and points to the limine fb with static lifetime and
        // pixels_per_row * height does not exceed the buffer size
        unsafe {
            core::ptr::write_bytes(
                self.fb_pointer,
                0u8,
                self.pixels_per_row * self.height
            );
        }
    }
}
