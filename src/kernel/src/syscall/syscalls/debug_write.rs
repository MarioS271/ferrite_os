// SPDX-License-Identifier: GPL-3.0-only
//! Debug Write Syscall Handler, writes a kernel log message at NOTICE severity
//!
//! Authors: MarioS271

use crate::knotice;
use crate::lib::types::user_slice::UserSlice;
use crate::syscall::result::{SyscallError, SyscallResult};

const BUFFER_SIZE: usize = 512;

/// Debug Write Syscall Handler
pub fn handler(user_slice: UserSlice) -> SyscallResult<()> {
    if user_slice.is_empty() {
        return Ok(());
    }

    let len = user_slice.len();
    if len > BUFFER_SIZE as u64 {
        return Err(SyscallError::InvalidArgument);
    }

    let mut buffer = [0u8; BUFFER_SIZE];
    user_slice.copy_to_slice(&mut buffer[..len as usize]);

    for byte in &mut buffer[..len as usize] {
        if !matches!(*byte, 0x20..=0x7e | b'\n') {
            *byte = b'?';
        }
    }

    let text = core::str::from_utf8(&buffer[..len as usize]).unwrap_or("(error while parsing raw utf8 to string)");
    knotice!("{}", text);

    Ok(())
}
