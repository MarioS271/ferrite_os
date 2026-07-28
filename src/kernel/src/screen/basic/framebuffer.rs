// SPDX-License-Identifier: GPL-3.0-only
//! Linear framebuffer wrapper for early-boot screen output.
//!
//! Authors: MarioS271

// Safe: kernel is single-threaded during early boot; raw pointer access is guarded by Once
unsafe impl Send for BasicFramebuffer {}
unsafe impl Sync for BasicFramebuffer {}

/// A thin wrapper around the Limine-provided linear framebuffer.
pub struct BasicFramebuffer {
    /// Pointer to the first pixel; pixels are row-major, `bytes_per_row` per row.
    pub fb_pointer: *mut u32,
    /// Number of 32-bit pixels per row (Limine pitch / 4).
    pub bytes_per_row: u32,
    pub width: u64,
    pub height: u64,
}

impl BasicFramebuffer {
    /// Construct a `BasicFramebuffer` from the Limine framebuffer descriptor.
    pub fn new(limine_fb: &limine::framebuffer::Framebuffer) -> Self {
        Self {
            fb_pointer: limine_fb.address() as *mut u32,
            bytes_per_row: (limine_fb.pitch / 4) as u32,
            width: limine_fb.width,
            height: limine_fb.height,
        }
    }

    /// Write zero (black) to every pixel in the framebuffer.
    pub fn clear(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                // Safe because we're iterating inside the given fb bounds and only
                // changing memory there
                unsafe {
                    self.fb_pointer
                        .add(y as usize * self.bytes_per_row as usize + x as usize)
                        .write_volatile(0u32);
                }
            }
        }
    }
}
