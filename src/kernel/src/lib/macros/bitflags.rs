// SPDX-License-Identifier: GPL-3.0-only
//! Bitflags macro for creating bitwise flag definitions
//!
//! Authors: MarioS271

macro_rules! bitflags {
    ($(#[$meta:meta])* $name:ident, $inner:ty) => {
        $(#[$meta])*
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $name($inner);

        impl $name {
            /// Construct an empty bitflag
            pub const fn empty() -> Self {
                Self(0)
            }

            /// Construct a bitflag from the given u8
            pub const fn from_u8(value: u8) -> Self {
                Self(value)
            }

            /// Check if the bitflag contains a specific flag or flag combination
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            /// Returns the inner [`u8`] value of `self`
            pub const fn as_u8(&self) -> u8 {
                self.0
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;

            /// Apply the bitwise OR operation to the two values
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
        impl core::ops::BitOrAssign for $name {
            /// Mask the inner value using the given value and the bitwise OR operation
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0
            }
        }
        impl core::ops::BitAnd for $name {
            type Output = Self;

            /// Apply the bitwise AND operation to the two values
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
        impl core::ops::BitAndAssign for $name {
            /// Mask the inner value using the given value and the bitwise AND operation
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0
            }
        }
    };
}
pub(crate) use bitflags;
