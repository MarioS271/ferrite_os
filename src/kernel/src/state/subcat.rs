//! Marker trait for `KState` subcategory structs.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

/// Marker trait implemented by every subsystem struct stored in [`KState`].
///
/// Currently has no required methods; it serves as a type-system annotation
/// that a struct is intended to be a subsystem slot in [`KState`].
pub trait KStateSubCategory {

}
