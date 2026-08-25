// SPDX-License-Identifier: GPL-3.0-only
//! Type for virtual Addresses
//!
//! Authors: MarioS271

use core::fmt::{Display, Debug, UpperHex, LowerHex, Pointer};
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A type representing virtual addresses in the CPU's virtual address space
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Creates a new `VirtAddr` from a raw `u64` address
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Creates a new `VirtAddr` from a `*const T` or `*mut T` pointer
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self(ptr as u64)
    }

    /// Returns the address as a `u64`
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the address as a `usize`
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Returns the address as a `*const T` pointer
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Returns the address as a `*mut T` pointer
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
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
impl VirtAddr {
    /// Returns a [`x86_64::VirtAddr`] with the same address value
    pub fn as_x86_64(self) -> x86_64::VirtAddr {
        x86_64::VirtAddr::new(self.0)
    }

    /// Returns the index into the P1 page table (PT) for this virtual address, in range `0..512`
    pub fn p1_index(self) -> x86_64::structures::paging::PageTableIndex {
        x86_64::structures::paging::PageTableIndex::new(((self.0 >> 12) & 0x01FF) as u16)
    }

    /// Returns the index into the P2 page table (PD) for this virtual address, in range `0..512`
    pub fn p2_index(self) -> x86_64::structures::paging::PageTableIndex {
        x86_64::structures::paging::PageTableIndex::new(((self.0 >> 21) & 0x01FF) as u16)
    }

    /// Returns the index into the P3 page table (PDPT) for this virtual address, in range `0..512`
    pub fn p3_index(self) -> x86_64::structures::paging::PageTableIndex {
        x86_64::structures::paging::PageTableIndex::new(((self.0 >> 30) & 0x01FF) as u16)
    }

    /// Returns the index into the P4 page table (PML4) for this virtual address, in range `0..512`
    pub fn p4_index(self) -> x86_64::structures::paging::PageTableIndex {
        x86_64::structures::paging::PageTableIndex::new(((self.0 >> 39) & 0x01FF) as u16)
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;

    /// `Add` trait for adding a `u64` to a `VirtAddr`, returns a new instance of `VirtAddr`
    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 + rhs)
    }
}
impl Add<VirtAddr> for VirtAddr {
    type Output = Self;

    /// `Add` trait for adding a `VirtAddr` to a `VirtAddr`, returns a new instance of `VirtAddr`
    fn add(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr::new(self.0 + rhs.0)
    }
}
impl AddAssign<u64> for VirtAddr {
    /// `AddAssign` trait for add-assigning a `u64` to a `VirtAddr`
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs
    }
}
impl AddAssign<VirtAddr> for VirtAddr {
    /// `AddAssign` trait for add-assigning a `VirtAddr` to a `VirtAddr`
    fn add_assign(&mut self, rhs: VirtAddr) {
        self.0 += rhs.0
    }
}

impl Sub<u64> for VirtAddr {
    type Output = Self;

    /// `Sub` trait for subtracting a `u64` from a `VirtAddr`, returns a new instance of `VirtAddr`
    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 - rhs)
    }
}
impl Sub<VirtAddr> for VirtAddr {
    type Output = Self;

    /// `Sub` trait for subtracting a `VirtAddr` from a `VirtAddr`, returns a new instance of `VirtAddr`
    fn sub(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr::new(self.0 - rhs.0)
    }
}
impl SubAssign<u64> for VirtAddr {
    /// `SubAssign` trait for sub-assigning a `u64` from a `VirtAddr`
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs
    }
}
impl SubAssign<VirtAddr> for VirtAddr {
    /// `SubAssign` trait for sub-assigning a `VirtAddr` from a `VirtAddr`
    fn sub_assign(&mut self, rhs: VirtAddr) {
        self.0 -= rhs.0
    }
}

impl Display for VirtAddr {
    /// Formats the address as `0x<hex>` (e.g. `0xffff800000000000`)
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}
impl Debug for VirtAddr {
    /// Formats the address as `VirtAddr(0x<hex>)` (e.g. `VirtAddr(0xffff800000000000)`)
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtAddr({:#x})", self.0)
    }
}
impl LowerHex for VirtAddr {
    /// Delegates to the inner `u64` for `{:x}` / `{:#x}` to work
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}
impl UpperHex for VirtAddr {
    /// Delegates to the inner `u64` for `{:X}` / `{:#X}` to work
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}
impl Pointer for VirtAddr {
    /// Delegates to the inner `u64`, cast to `*const ()` for `{:p}` to work
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Pointer::fmt(&(self.0 as *const ()), f)
    }
}
