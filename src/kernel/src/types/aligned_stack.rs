// SPDX-License-Identifier: GPL-3.0-only
//! 16-byte-aligned stack backing storage.
//!
//! Authors: MarioS271

/// An `N`-byte array aligned to a 16-byte boundary, for ABI-compliant stack storage.
#[repr(align(16))]
pub struct AlignedStack<const N: usize> {
    pub array: [u8; N]
}
