// SPDX-License-Identifier: GPL-3.0-only
//! Root scheduler data struct
//!
//! Authors: MarioS271

use crate::sched::state::procs::Procs;

/// Holds the scheduler state, including active process data, scheduler info and more
pub struct Sched {
    procs: Procs
}

impl Sched {
    /// Constructor; initializes all values zeroed or uninited
    pub const fn new() -> Self {
        Self {
            procs: Procs::new()
        }
    }

    /// Getter for `Procs::procs`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn procs(&self) -> &Procs {
        &self.procs
    }
}
