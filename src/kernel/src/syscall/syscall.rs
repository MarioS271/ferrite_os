// SPDX-License-Identifier: GPL-3.0-only
//! Syscall table
//!
//! Authors: MarioS271

use crate::lib::types::user_slice::UserSlice;
use super::args::SyscallArgs;
use super::result::ToSyscallU64;
use super::result::{SyscallError, SyscallResult};

/// A type which holds all possible syscall variants and their numeric equivalents
#[allow(dead_code)]
#[repr(u64)]
pub enum Syscall {
    Exit = 0,
    DebugWrite = 1
}

impl Syscall {
    /// Build a new [`Syscall`] from a `u64`
    pub fn from_syscall_num(num: u64) -> Option<Self> {
        use Syscall::*;
        match num {
            0 => Some(Exit),
            1 => Some(DebugWrite),
            _ => None
        }
    }

    /// Convert the given [`Syscall`] into a `&str`, returns "Unknown" if the given syscall
    /// is not valid (like passing [`Syscall::MAX`])
    pub fn as_str(&self) -> &'static str {
        use Syscall::*;
        match self {
            Exit => "Exit",
            DebugWrite => "DebugWrite",
            _ => "Unknown"
        }
    }

    /// Dispatch to the correct syscall via the given [`SyscallArgs`]
    pub fn dispatch(args: &SyscallArgs) -> SyscallResult<u64> {
        let Some(syscall) = Syscall::from_syscall_num(args.syscall_num) else {
            #[cfg(feature = "syscall-debug-logging")]
            crate::kdebug!("Received Invalid Syscall #{}", args.syscall_num);

            return Err(SyscallError::InvalidSyscall);
        };

        #[cfg(feature = "syscall-debug-logging")]
        crate::kdebug!("Received Syscall {} (#{})", syscall.as_str(), args.syscall_num);

        use super::syscalls;
        let res = match syscall {
            Syscall::Exit => {
                syscalls::exit::handler(args.arg1 as u32)
            },
            Syscall::DebugWrite => {
                let user_slice = UserSlice::new(args.arg1, args.arg2)?;
                syscalls::debug_write::handler(user_slice).map(|r| r.as_syscall_u64())
            }
        };

        res
    }
}
