// SPDX-License-Identifier: GPL-3.0-only
//! 16-byte-aligned stack backing storage.
//!
//! Authors: MarioS271

use crate::types::addr::VirtAddr;

/// An `N`-byte array aligned to a 16-byte boundary, for ABI-compliant stack storage.
#[repr(align(16))]
pub struct AlignedStack<const N: usize>([u8; N]);

impl<const N: usize> AlignedStack<N> {
    /// Constructor; creates a new zeroed [`AlignedStack`] of size N
    pub const fn new() -> Self {
        Self([0u8; N])
    }

    /// Getter for the inner array
    pub fn get(&self) -> [u8; N] {
        self.0
    }

    /// Returns the top of the stack as a [`VirtAddr`]
    pub fn get_stack_top(&self) -> VirtAddr {
        // Safety: .add(len) computes a one-past-the-end pointer, which is valid per rust's pointer rules
        unsafe { VirtAddr::from_ptr(self.0.as_ptr().add(self.0.len())) }
    }
}
