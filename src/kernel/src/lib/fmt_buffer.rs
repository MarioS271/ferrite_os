//! lib/fmt_buffer.rs
//! Buffer which can read in format strings
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

pub struct FmtBuffer<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> FmtBuffer<N> {
    pub fn new() -> Self {
        Self { buf: [0u8; N], pos: 0 }
    }

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
