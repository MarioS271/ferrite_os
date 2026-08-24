// SPDX-License-Identifier: GPL-3.0-only
//! Type for physical addresses
//!
//! Authors: MarioS271

use core::ops::{Add, AddAssign, Sub, SubAssign};
use crate::types::addr::VirtAddr;

/// A type representing physical addresses in the CPU's physical address space
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Creates a new `PhysAddr` from a raw `u64` address
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Returns the address as a `u64`
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the address as a `usize`
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Returns a new address aligned to the next larger address which is aligned to `align`
    ///
    /// # Panics
    /// In debug builds if `align` is not a power of two
    pub fn align_up(self, align: u64) -> Self {
        debug_assert!(align.is_power_of_two());
        Self((self.0 + (align - 1)) & !(align - 1))
    }

    /// Returns a new address aligned to the next smaller address which is aligned to `align`
    ///
    /// # Panics
    /// In debug builds if `align` is not a power of two
    pub fn align_down(self, align: u64) -> Self {
        debug_assert!(align.is_power_of_two());
        Self(self.0 & !(align - 1))
    }

    /// Check whether the address is aligned to `align`
    pub fn is_aligned(self, align: u64) -> bool {
        self.0 & (align - 1) == 0
    }
}

#[cfg(target_arch = "x86_64")]
impl PhysAddr {
    /// Returns a [`x86_64::PhysAddr`] with the same address value
    pub fn as_x86_64(self) -> x86_64::PhysAddr {
        x86_64::PhysAddr::new(self.0)
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;

    /// `Add` trait for adding a `u64` to a `PhysAddr`, returns a new instance of `PhysAddr`
    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0 + rhs)
    }
}
impl Add<PhysAddr> for PhysAddr {
    type Output = Self;

    /// `Add` trait for adding a `PhysAddr` to a `PhysAddr`, returns a new instance of `PhysAddr`
    fn add(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr::new(self.0 + rhs.0)
    }
}
impl AddAssign<u64> for PhysAddr {
    /// `AddAssign` trait for add-assigning a `u64` to a `PhysAddr`
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs
    }
}
impl AddAssign<PhysAddr> for PhysAddr {
    /// `AddAssign` trait for add-assigning a `PhysAddr` to a `PhysAddr`
    fn add_assign(&mut self, rhs: PhysAddr) {
        self.0 += rhs.0
    }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;

    /// `Sub` trait for subtracting a `u64` from a `PhysAddr`, returns a new instance of `PhysAddr`
    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0 - rhs)
    }
}
impl Sub<PhysAddr> for PhysAddr {
    type Output = Self;

    /// `Sub` trait for subtracting a `PhysAddr` from a `PhysAddr`, returns a new instance of `PhysAddr`
    fn sub(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr::new(self.0 - rhs.0)
    }
}
impl SubAssign<u64> for PhysAddr {
    /// `SubAssign` trait for sub-assigning a `u64` from a `PhysAddr`
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs
    }
}
impl SubAssign<PhysAddr> for PhysAddr {
    /// `SubAssign` trait for sub-assigning a `PhysAddr` from a `PhysAddr`
    fn sub_assign(&mut self, rhs: PhysAddr) {
        self.0 -= rhs.0
    }
}
