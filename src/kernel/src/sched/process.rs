// SPDX-License-Identifier: GPL-3.0-only
//! The definition of a process (its data and state)
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::mm::vmm::address_space::AddressSpace;
use core::borrow::Borrow;
use core::sync::atomic::AtomicI32;

/// Type which represents a PID
pub type Pid = i32;

/// Type which represents an atomic PID
pub type AtomicPid = AtomicI32;

/// Represents all necessary data to manage processes
pub struct Process {
    pub pid: Pid,
    pub parent_pid: Pid,
    pub addr_space: AddressSpace,
    pub status: ProcessStatus,
    pub kernel_stack: KernelStack,
    pub regs: SavedRegs
}
impl Borrow<Pid> for Process {
    /// Returns a reference to [`Process::pid`] to make it possible for [`BTreeSet`] to
    /// compare it with a [`Pid`] directly
    fn borrow(&self) -> &Pid {
        &self.pid
    }
}
impl PartialEq for Process {
    /// Only checks equality for [`Process::pid`] and no other property
    fn eq(&self, other: &Self) -> bool {
        self.pid.eq(&other.pid)
    }
}
impl Eq for Process {}
impl PartialOrd for Process {
    /// Delegates to [`Process::cmp`]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Process {
    /// Compares only [`Process::pid`] and no other property
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
