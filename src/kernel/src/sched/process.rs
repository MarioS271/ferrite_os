// SPDX-License-Identifier: GPL-3.0-only
//! The definition of a process (its data and state)
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::mm::vmm::address_space::AddressSpace;
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
    pub kernel_stack_top: VirtAddr,
    pub regs: SavedRegs
}
impl Process {
    /// Create a new process which owns an address space
    pub fn new(pid: Pid, parent_pid: Pid, addr_space: AddressSpace, kernel_stack_top: VirtAddr) -> Self {
        Self {
            pid,
            parent_pid,
            addr_space,
            status: ProcessStatus::Ready,
            kernel_stack_top,
            regs: SavedRegs::new()
        }
    }
}

/// The status of a process
pub enum ProcessStatus {
    /// Ready to be run by the scheduler
    Ready,
    /// Currently running on a CPU
    Running,
    /// Waiting on a resource (I/O, a lock, ...)
    Waiting,
    /// Exited; also contains an exit code
    Zombie(i32)
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
impl SavedRegs {
    /// Construct a zeroed (except `rflags`) [`SavedRegs`] instance
    ///
    /// `rflags` gets the reserved bit 1, which the CPU requires to be set,
    /// plus IF so the process can be preempted
    pub const fn new() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rsp: 0, rflags: 0x202
        }
    }
}
