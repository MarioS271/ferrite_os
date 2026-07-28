//! Fixed-capacity formatting buffer for no-alloc contexts.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

/// A stack-allocated, fixed-size buffer that implements [`core::fmt::Write`].
///
/// Intended for interrupt handlers and the panic path where heap allocation is
/// unavailable. Create one with `FmtBuffer::<N>::new()`, write into it using the
/// `write!` macro, then read the result back via `as_str()`.
///
/// If more bytes are written than the buffer can hold, the excess is silently
/// dropped. `write_str` always returns `Ok` so that `write!` does not abort
/// early — callers should size `N` large enough for their expected output.
pub struct FmtBuffer<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> FmtBuffer<N> {
    /// Create an empty buffer with all bytes zeroed.
    pub fn new() -> Self {
        Self { buf: [0u8; N], pos: 0 }
    }

    /// Return the bytes written so far as a `&str`.
    ///
    /// Returns an empty string if the buffer contains invalid UTF-8 (should not
    /// happen in practice since `write_str` receives `&str` slices).
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
