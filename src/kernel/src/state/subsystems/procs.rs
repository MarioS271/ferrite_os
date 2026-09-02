// SPDX-License-Identifier: GPL-3.0-only
//! Process table placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::mem::vmm::address_space::AddressSpace;
use crate::types::addr::VirtAddr;
use crate::types::irq_mutex::IrqMutex;
use alloc::collections::BTreeSet;
use core::borrow::Borrow;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicI32, Ordering};

/// Holds the kernel process table mapping PIDs to process control blocks.
pub struct Procs {
    procs: UnsafeCell<MaybeUninit<IrqMutex<BTreeSet<ProcessData>>>>,
    active_pid: AtomicPid
}

/// Safety:
/// - `procs` is written to exactly once before concurrent access can happen;
///   after [`Procs::init_procs`] no mutable reference/pointer is ever created again
/// - `active_pid` is an [`AtomicPid`] and therefore is already [`Sync`]
unsafe impl Sync for Procs {}

impl Procs {
    /// Constructor; initializes all values zeroed or uninited
    pub const fn new() -> Self {
        Self {
            procs: UnsafeCell::new(MaybeUninit::uninit()),
            active_pid: AtomicPid::new(0)
        }
    }

    /// Initialize an empty [`BTreeSet`] inside `Procs::procs`
    ///
    /// # Safety
    /// This method derefs `self.procs.get()`.
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - There must be no mutable references or pointers to `self.procs`
    /// - This method must be called exactly once
    pub unsafe fn init_procs(&self) {
        unsafe { (*self.procs.get()).write(IrqMutex::new(BTreeSet::new())); }
    }

    /// Set the value of `Procs::active_pid`
    pub fn set_active_pid(&self, new_active_pid: Pid) {
        self.active_pid.store(new_active_pid, Ordering::Release);
    }

    /// Getter for `Procs::procs`
    ///
    /// # Safety
    /// This method derefs `self.procs.get()` and calls [`MaybeUninit::assume_init_ref()`].
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - `self.procs` must have been initialized first (via `init_procs`)
    /// - There must be no mutable references or pointers to `self.procs`
    pub unsafe fn procs(&self) -> &IrqMutex<BTreeSet<ProcessData>> {
        unsafe { (*self.procs.get()).assume_init_ref() }
    }

    /// Getter for `Procs::active_pid`
    pub fn active_pid(&self) -> Pid {
        self.active_pid.load(Ordering::Acquire)
    }
}

/// Type which represents a PID
pub type Pid = i32;
/// Type which represents an atomic PID
pub type AtomicPid = AtomicI32;

/// Represents all necessary data to manage processes
pub struct ProcessData {
    pub pid: Pid,
    pub parent_pid: Pid,
    pub addr_space: AddressSpace,
    pub status: ProcessStatus,
    pub kernel_stack: KernelStack,
    pub regs: SavedRegs
}
impl Borrow<Pid> for ProcessData {
    /// Returns a reference to [`ProcessData::pid`] to make it possible for [`BTreeSet`] to
    /// compare it with a [`Pid`] directly
    fn borrow(&self) -> &Pid {
        &self.pid
    }
}
impl PartialEq for ProcessData {
    /// Only checks equality for [`ProcessData::pid`] and no other property
    fn eq(&self, other: &Self) -> bool {
        self.pid.eq(&other.pid)
    }
}
impl Eq for ProcessData {}
impl PartialOrd for ProcessData {
    /// Delegates to [`ProcessData::cmp`]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ProcessData {
    /// Compares only [`ProcessData::pid`] and no other property
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.pid.cmp(&other.pid)
    }
}

/// The status of a process
pub enum ProcessStatus {
    Running,
    Zombie(i32)
}

/// The kernel stack of a process (start address and size)
pub struct KernelStack {
    pub base: VirtAddr,
    pub size: u64
}

/// The saved CPU registers for when execution was paused by the scheduler
#[cfg(target_arch = "x86_64")]
pub struct SavedRegs {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64,
    pub r8:  u64, pub r9:  u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub rip: u64, pub rsp: u64, pub rflags: u64,
}
