// SPDX-License-Identifier: GPL-3.0-only
//! Syscall table
//!
//! Authors: MarioS271

use crate::syscall::error::{SyscallError, SyscallResult};
use crate::syscall::structs::SyscallFrame;

/// A table which holds all possible syscall variants
#[repr(u64)]
pub enum Syscall {
    Exit = 0,
    MAX = 1
}

impl Syscall {
    pub fn is_valid_syscall(num: u64) -> bool {
        if num < 0 || num >= Syscall::MAX as u64 {
            return false;
        }
        true
    }

    pub fn from_syscall_num(num: u64) -> Option<Self> {
        use Syscall::*;
        match num {
            0 => Some(Exit),
            _ => None
        }
    }

    pub fn as_str(&self) -> &str {
        use Syscall::*;
        match self {
            Exit => "Exit",
            _ => "Unknown"
        }
    }

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

        Ok(0u64)
    }
}
