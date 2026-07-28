// SPDX-License-Identifier: GPL-3.0-only
//! 16-byte-aligned stack backing storage.
//!
//! Authors: MarioS271

/// A `N`-byte array guaranteed to start on a 16-byte boundary.
///
/// The x86-64 System V ABI requires that the stack pointer is 16-byte aligned
/// before a `call` instruction. IST stacks in the TSS are addressed by their
/// *top* (highest address, since stacks grow down), so the stored top pointer
/// must also satisfy this alignment. `#[repr(align(16))]` ensures the underlying
/// array's first byte — and therefore any address within it — is properly aligned.
#[repr(align(16))]
pub struct AlignedStack<const N: usize> {
    pub array: [u8; N]
}
