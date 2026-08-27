// SPDX-License-Identifier: GPL-3.0-only
//! VMM definitions (VMM struct, common helpers)
//!
//! Authors: MarioS271

/// Namespace for all VMM methods
pub struct Vmm;

/// Error type for VMM operations
pub enum VmmError {
    VmaOverlap
}
