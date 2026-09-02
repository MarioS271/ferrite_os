// SPDX-License-Identifier: GPL-3.0-only
//! Linear framebuffer wrapper for early-boot screen output.
//!
//! Authors: MarioS271

/// A thin wrapper around the bootloader-provided framebuffer
pub struct BasicFramebuffer {
    pub fb_pointer: *mut u32,
    pub bytes_per_row: u32,
    pub width: u64,
    pub height: u64,
}

/// Safety: [`BasicFramebuffer`] lives inside an [`IrqMutex`](crate::types::irq_mutex::IrqMutex)
unsafe impl Sync for BasicFramebuffer {}
/// Safety: `BasicFrameBuffer::fb_pointer` points to the valid limine-allocated fb which has a static lifetime
unsafe impl Send for BasicFramebuffer {}

impl BasicFramebuffer {
    /// Constructor; returns a `BasicFramebuffer` constructed from limine's [`Framebuffer`](limine::framebuffer::Framebuffer)
    pub fn new(limine_fb: &limine::framebuffer::Framebuffer) -> Self {
        Self {
            fb_pointer: limine_fb.address() as *mut u32,
            bytes_per_row: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
        }
    }

    /// Write zero (black) to every pixel in the framebuffer
    pub fn clear(&self) {
        // Safety: fb_pointer is valid and points to the limine fb with static lifetime and
        // bytes_per_row * height does not exceed the buffer size
        unsafe {
            core::ptr::write_bytes(self.fb_pointer, 0u8, (self.bytes_per_row as u64 * self.height) as usize);
        }
    }
}
