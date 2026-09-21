// SPDX-License-Identifier: GPL-3.0-only
//! Syscall table
//!
//! Authors: MarioS271

use crate::syscall::error::{SyscallError, SyscallResult};
use crate::syscall::structs::SyscallFrame;

/// A type which holds all possible syscall variants and their numeric equivalents
#[allow(dead_code)]
#[repr(u64)]
pub enum Syscall {
    Exit = 0,
    MAX = 1
}

impl Syscall {
    /// Build a new [`Syscall`] from a `u64`
    pub fn from_syscall_num(num: u64) -> Option<Self> {
        use Syscall::*;
        match num {
            0 => Some(Exit),
            _ => None
        }
    }

    /// Check whether the given `u64` is a valid syscall number
    pub fn is_valid_syscall(num: u64) -> bool {
        if num >= Syscall::MAX as u64 {
            return false;
        }
        true
    }

    /// Convert the given [`Syscall`] into a `&str`, returns "Unknown" if the given syscall
    /// is not valid (like passing [`Syscall::MAX`])
    pub fn as_str(&self) -> &str {
        use Syscall::*;
        match self {
            Exit => "Exit",
            _ => "Unknown"
        }
    }

    /// Dispatch to the correct syscall via the given [`SyscallFrame`]
    pub fn dispatch(frame: &SyscallFrame) -> SyscallResult<u64> {
        if !Syscall::is_valid_syscall(frame.syscall_num) {
            #[cfg(feature = "syscall-debug-logging")]
            crate::kdebug!("Received Invalid Syscall #{}", frame.syscall_num);

            return Err(SyscallError::InvalidSyscall);
        }

        #[cfg(feature = "syscall-debug-logging")]
        {
            let syscall = Syscall::from_syscall_num(frame.syscall_num).unwrap();
            crate::kdebug!("Received Syscall {} (#{})", syscall.as_str(), frame.syscall_num);
        }

        use super::syscalls;
        match Syscall::from_syscall_num(frame.syscall_num).unwrap() {
            Syscall::Exit => syscalls::exit::handler(frame.arg1 as u32),
            _ => unreachable!()
        }

        // TODO: return actual val
        Ok(0u64)
    }
}
