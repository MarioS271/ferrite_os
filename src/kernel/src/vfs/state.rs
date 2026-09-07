// SPDX-License-Identifier: GPL-3.0-only
//! Virtual filesystem state placeholder for [`KState`].
//!
//! Authors: MarioS271

/// Holds the virtual filesystem layer state and registered filesystem drivers
#[repr(align(64))]
pub struct Vfs {

}

impl Vfs {
    /// Constructor; initializes all values zeroed or uninited
    pub const fn new() -> Self {
        Self {

        }
    }
}
