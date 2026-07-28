// SPDX-License-Identifier: GPL-3.0-only
//! Linear framebuffer wrapper for early-boot screen output.
//!
//! Authors: MarioS271

// Safe: kernel is single-threaded during early boot; raw pointer access is guarded by Once
unsafe impl Send for BasicFramebuffer {}
unsafe impl Sync for BasicFramebuffer {}

/// A thin wrapper around the Limine-provided linear framebuffer.
///
/// Stores the base pointer and dimensions needed to write pixels. Each pixel is a
/// 32-bit value in the format the display controller expects (typically `0x00RRGGBB`
/// or `0x00BBGGRR` depending on the framebuffer's pixel format). Callers compute
/// pixel offsets as `y * bytes_per_row + x` and write via `fb_pointer.add(offset)`.
pub struct BasicFramebuffer {
    /// Pointer to the first pixel of the framebuffer. Pixels are laid out in row-major
    /// order with `bytes_per_row` 32-bit values per row.
    pub fb_pointer: *mut u32,
    /// Number of 32-bit pixels per row. Derived from Limine's `pitch / 4` (pitch is
    /// bytes per scanline; dividing by 4 converts to 32-bit pixel units).
    pub bytes_per_row: u32,
    /// Width of the framebuffer in pixels.
    pub width: u64,
    /// Height of the framebuffer in pixels.
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
    ///
    /// Uses `write_volatile` so the compiler cannot elide the writes even if it
    /// determines the values are never read by Rust code.
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
