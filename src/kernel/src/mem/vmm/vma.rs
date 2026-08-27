// SPDX-License-Identifier: GPL-3.0-only
//! Type that represents virtual memory areas which are used to mark certain memory read, write, user or execute
//!
//! Authors: MarioS271

use crate::types::addr::VirtAddr;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr};

/// A type representing virtual memory areas
/// > **Important**: `end_addr` is exclusive
pub struct Vma {
    pub start_addr: VirtAddr,
    pub end_addr: VirtAddr,
    pub flags: VmaFlags
}

impl Borrow<VirtAddr> for Vma {
    /// Returns a reference to [`Vma::start_addr`] to make it possible for `BTreeSet` to compare it with a `VirtAddr` directly
    fn borrow(&self) -> &VirtAddr {
        &self.start_addr
    }
}
impl PartialEq<Self> for Vma {
    /// Only checks equality for [`Vma::start_addr`] and no other property
    fn eq(&self, other: &Self) -> bool {
        self.start_addr.eq(&other.start_addr)
    }
}
impl Eq for Vma {}
impl Ord for Vma {
    /// Compares only [`Vma::start_addr`] and no other property
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_addr.cmp(&other.start_addr)
    }
}
impl PartialOrd for Vma {
    /// Delegates to [`Vma::cmp`]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Flags to describe the properties of a VMA
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VmaFlags(u8);

impl VmaFlags {
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXEC: Self = Self(1 << 2);
    pub const USER: Self = Self(1 << 3);

    /// Constructor that returns an empty/zeroed `VmaFlags`
    pub fn empty() -> Self {
        Self(0)
    }

    /// Check whether the given `VmaFlags` inner value contains all the flags in `other`
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for VmaFlags {
    type Output = Self;

    /// Apply the bitwise OR operation to the two values
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl BitAnd for VmaFlags {
    type Output = Self;

    /// Apply the bitwise AND operation to the two values
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
