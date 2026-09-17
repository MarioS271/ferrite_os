// SPDX-License-Identifier: GPL-3.0-only
//! MM Error Type
//!
//! Authors: MarioS271

use crate::mm::vmm::VmmError;

/// A wrapper around `Result<T, MmError>`
pub type MmResult<T> = Result<T, MmError>;

/// An enum which describes possible memory management errors that can occur
#[derive(Debug)]
pub enum MmError {
    /// No free memory left to allocate
    OutOfMemory,
    /// The given address is not frame aligned
    MisalignedAddress,
    /// The requested region overlaps with an existing one
    Overlap,
    /// No region at the given address found
    NotFound
}
impl From<VmmError> for MmError {
    /// Convert a given [`VmmError`] to a [`MmError`]
    fn from(value: VmmError) -> Self {
        match value {
            VmmError::OutOfMemory => Self::OutOfMemory,
            VmmError::VmaOverlap => Self::Overlap,
            VmmError::VmaNotFound => Self::NotFound,
            VmmError::InvalidUnmap | VmmError::InvalidRemap => Self::NotFound
        }
    }
}
