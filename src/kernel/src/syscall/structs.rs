// SPDX-License-Identifier: GPL-3.0-only
//! Syscall Frame
//!
//! Authors: MarioS271

pub struct SyscallFrame {
    pub syscall_num: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
    pub arg6: u64
}
