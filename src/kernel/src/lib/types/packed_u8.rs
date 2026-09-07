// SPDX-License-Identifier: GPL-3.0-only
//! Packed u8; store multiple values shorter than 8 bits in one byte
//!
//! Authors: MarioS271

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PackedU8(u8);

impl PackedU8 {
    /// Get `bits` bits starting from bit `start`
    pub fn get(self, start: u8, bits: u8) -> u8 {
        (self.0 >> start) & ((1 << bits) - 1)
    }

    /// Set `bits` bits starting from bit `start` to `value`
    /// `value` will be shortened to `bits` bits to avoid overwriting other data
    pub fn set(&mut self, start: u8, bits: u8, value: u8) {
        let mask = ((1 << bits) - 1) << start;
        self.0 = (self.0 & !mask) | ((value << start) & mask);
    }
}

impl Default for PackedU8 {
    fn default() -> Self {
        Self(0)
    }
}
