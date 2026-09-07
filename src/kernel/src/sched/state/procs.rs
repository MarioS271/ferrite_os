// SPDX-License-Identifier: GPL-3.0-only
//! Process Data
//!
//! Authors: MarioS271

use crate::lib::sync::irq_mutex::IrqMutex;
use crate::lib::sync::unchecked_cell::UncheckedCell;
use crate::sched::process::{AtomicPid, Pid, Process};
use alloc::collections::BTreeSet;
use core::sync::atomic::Ordering;

/// Holds the kernel process table mapping PIDs to process control blocks.
pub struct Procs {
    procs: UncheckedCell<IrqMutex<BTreeSet<Process>>>,
    active_pid: AtomicPid
}

impl Procs {
    /// Constructor; initializes all values zeroed or uninited
    pub const fn new() -> Self {
        Self {
            procs: UncheckedCell::new(),
            active_pid: AtomicPid::new(0)
        }
    }

    /// Initialize an empty [`BTreeSet`] inside `Procs::procs`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_procs(&self) {
        unsafe { self.procs.init(IrqMutex::new(BTreeSet::new())); }
    }

    /// Set the value of `Procs::active_pid`
    pub fn set_active_pid(&self, new_active_pid: Pid) {
        self.active_pid.store(new_active_pid, Ordering::Release);
    }

    /// Getter for `Procs::procs`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn procs(&self) -> &IrqMutex<BTreeSet<Process>> {
        unsafe { self.procs.get() }
    }

    /// Getter for `Procs::active_pid`
    pub fn active_pid(&self) -> Pid {
        self.active_pid.load(Ordering::Acquire)
    }
}
