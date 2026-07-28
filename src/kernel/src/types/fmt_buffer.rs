// SPDX-License-Identifier: GPL-3.0-only
//! Fixed-capacity formatting buffer for no-alloc contexts.
//!
//! Authors: MarioS271

/// Stack-allocated, fixed-size [`core::fmt::Write`] buffer for no-alloc contexts
/// (interrupt handlers, panic path). Writes beyond capacity `N` are silently dropped.
pub struct FmtBuffer<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> FmtBuffer<N> {
    /// Create an empty buffer with all bytes zeroed.
    pub fn new() -> Self {
        Self { buf: [0u8; N], pos: 0 }
    }

    /// Return the bytes written so far as a `&str` (empty on invalid UTF-8).
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }
}

impl<const N: usize> core::fmt::Write for FmtBuffer<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = N - self.pos;
        let to_copy = bytes.len().min(remaining);
        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;
        Ok(())
    }
}
