//! types/aligned_stack.rs
//! Struct representing a 16-byte aligned stack of size N
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

#[repr(align(16))]
pub struct AlignedStack<const N: usize> {
    pub array: [u8; N]
}