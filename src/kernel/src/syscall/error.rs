// SPDX-License-Identifier: GPL-3.0-only
//! Syscall Error Return Codes
//!
//! Authors: MarioS271

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
    InvalidArgument = 2
}
