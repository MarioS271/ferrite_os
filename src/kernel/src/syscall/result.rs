// SPDX-License-Identifier: GPL-3.0-only
//! Syscall Result Conversion and Error Types
//!
//! Authors: MarioS271

/// A trait to convert the implementing type to a valid syscall result `u64`
pub trait ToSyscallU64 {
    /// Convert the given `self` into a `u64`
    fn as_syscall_u64(&self) -> u64;
}

impl ToSyscallU64 for () {
    /// Always returns `0u64`
    fn as_syscall_u64(&self) -> u64 {
        0u64
    }
}


/// Wrapper type for `Result<T, SyscallError>`
pub type SyscallResult<T> = Result<T, SyscallError>;

/// A trait to implement methods into the [`SyscallResult`] type
pub trait SyscallResultTrait {
    /// Convert a given [`SyscallError`] into a `u32`
    fn as_u32(&self) -> u32;
}
impl<T> SyscallResultTrait for SyscallResult<T> {
    fn as_u32(&self) -> u32 {
        match self {
            Ok(_) => 0,
            Err(e) => *e as u32
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum SyscallError {
    InvalidSyscall = 1,
    InvalidArgument = 2,
    InternalError = 3,
    BadAddress = 4
}
