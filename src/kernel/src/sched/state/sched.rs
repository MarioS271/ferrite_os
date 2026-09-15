// SPDX-License-Identifier: GPL-3.0-only
//! Root scheduler data struct
//!
//! Authors: MarioS271

use crate::lib::sync::irq_mutex::IrqMutex;
use crate::lib::sync::unchecked_cell::UncheckedCell;
use crate::sched::process::{AtomicPid, Pid, Process};
use alloc::collections::BTreeMap;
use core::sync::atomic::Ordering;

/// Wrapper type for `BTreeMap<Pid, Process`
pub type ProcessMap = BTreeMap<Pid, Process>;

/// Holds the scheduler state, including active process data, scheduler info and more
pub struct Sched {
    procs: UncheckedCell<IrqMutex<ProcessMap>>,
    active_pid: AtomicPid,
    next_pid: AtomicPid
}

impl Sched {
    /// Constructor; initializes all values zeroed
    pub const fn new() -> Self {
        Self {
            procs: UncheckedCell::new(),
            active_pid: AtomicPid::new(0),
            next_pid: AtomicPid::new(0)
        }
    }

    /// Initialize the sched state struct to its correct default values
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub fn init(&self) {
        unsafe { self.procs.init(IrqMutex::new(BTreeMap::new())); }
        self.next_pid.store(1, Ordering::Relaxed);
    }

    /// Hand out the next free PID
    pub fn alloc_pid(&self) -> Pid {
        self.next_pid.fetch_add(1, Ordering::Relaxed)
    }

    /// Getter for `Sched::active_pid`
    pub fn active_pid(&self) -> Pid {
        self.active_pid.load(Ordering::Acquire)
    }

    /// Set the value of `Sched::active_pid`
    pub fn set_active_pid(&self, new_active_pid: Pid) {
        self.active_pid.store(new_active_pid, Ordering::Release);
    }

    /// Run `f` with the process with belongs to `pid`
    ///
    /// # Important
    /// The process table is held for the entire closure. NEVER touch user memory inside `f`
    /// as that could cause a page fault which would attempt to lock the process table, resulting
    /// in a deadlock. Copy necessary values and then modify after the closure returns.
    ///
    /// # Safety
    /// The caller must guarantee that [`Sched::init_procs`] has already been called
    pub unsafe fn with_process<R>(&self, pid: Pid, f: impl FnOnce(&mut Process) -> R) -> Option<R> {
        let mut procs = unsafe { self.procs.get().lock() };
        procs.get_mut(&pid).map(f)
    }

    /// Run `f` with the process which is currently running
    ///
    /// # Safety
    /// The caller must guarantee that [`Sched::init_procs`] has already been called
    pub unsafe fn with_active_process<R>(&self, f: impl FnOnce(&mut Process) -> R) -> Option<R> {
        unsafe { self.with_process(self.active_pid(), f) }
    }

    /// Add a process to the process table
    ///
    /// # Safety
    /// The caller must guarantee that [`Sched::init_procs`] has already been called
    pub unsafe fn insert_process(&self, process: Process) {
        unsafe { self.procs.get() }.lock().insert(process.pid, process);
    }

    /// Remove a process from the process table and return it, or `None` if it did not exist
    ///
    /// # Safety
    /// The caller must guarantee that [`Sched::init_procs`] has already been called
    pub unsafe fn remove_process(&self, pid: Pid) -> Option<Process> {
        unsafe { self.procs.get() }.lock().remove(&pid)
    }
}
