// SPDX-License-Identifier: GPL-3.0-only
//! Untrusted userspace slice
//!
//! Authors: MarioS271

use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::fmt_buffer::FmtBuffer;
use crate::mm::layout::USER_MAX;
use crate::syscall::result::{SyscallError, SyscallResult};

/// Data structure to store fat pointers provided by userspace
#[derive(Copy, Clone, Debug)]
pub struct UserSlice {
    address: u64,
    length: u64
}

impl UserSlice {
    /// Create a new [`UserSlice`] from the given params
    ///
    /// The returned `Ok(Self)` is guaranteed to be fully below `USER_MAX`
    ///
    /// # Errors
    /// Returns `SyscallError::BadAddress` if the given memory region isn't fully below `USER_MAX`
    /// or if it overflows
    pub fn new(address: u64, length: u64) -> SyscallResult<Self> {
        let Some(end) = address.checked_add(length) else {
            return Err(SyscallError::BadAddress)
        };
        if end > USER_MAX {
            return Err(SyscallError::BadAddress)
        }

        Ok(Self {
            address,
            length
        })
    }

    /// Getter for `self.length`
    pub fn len(&self) -> u64 {
        self.length
    }

    /// Check whether the given `self` is empty (zero-length)
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Copies the userspace data into the given `slice`
    ///
    /// # Panics
    /// - `slice` is not exactly as large as this [`UserSlice`] (only with the `debug-checks` feature)
    /// - any of the cases listed on [`Self::partial_copy_to_slice`]
    pub fn copy_to_slice(&self, slice: &mut [u8]) {
        #[cfg(feature = "debug-checks")]
        if self.length != slice.len() as u64 {
            self.incorrect_size_panic(slice.len(), self.length);
        }

        self.partial_copy_to_slice(slice, 0, self.length)
    }

    /// Copies a part of the given userspace data into the given `slice`
    ///
    /// # Panics
    /// - `offset + length` overflows
    /// - the requested range reaches past the end of this [`UserSlice`]
    /// - `slice` is smaller than `length`
    ///
    /// All of these indicate a kernel-side bug, never bad userspace input
    pub fn partial_copy_to_slice(&self, slice: &mut [u8], offset: u64, length: u64) {
        let Some(end) = offset.checked_add(length) else {
            kernel_panic(
                PanicCode::InternalKernelError,
                "UserSlice: adding offset and length overflowed"
            );
        };

        if end > self.length {
            kernel_panic(
                PanicCode::InternalKernelError,
                "UserSlice: requested range exceeds slice bounds"
            );
        }
        if length > slice.len() as u64 {
            self.incorrect_size_panic(slice.len(), length);
        }

        if length == 0 {
            return;
        }

        #[cfg(feature = "debug-checks")]
        if self.address.checked_add(offset).is_none() {
            kernel_panic(
                PanicCode::InternalKernelError,
                "UserSlice: adding address and offset overflowed"
            );
        }
        let start_addr = self.address.wrapping_add(offset);

        // Safety:
        // - self.address is under USER_MAX (guaranteed by Self::new)
        // - length of dst buffer is equal or larger than src buffer
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!(
                "rep movsb",

                inout("rsi") start_addr => _,
                inout("rdi") slice.as_mut_ptr() => _,
                inout("rcx") length => _,

                options(nostack, preserves_flags)
            );

            #[cfg(target_arch = "aarch64")]
            compile_error!("not implemented");
        }
    }

    /// Panic Message for when `dst_len` is not sized like `src_len` (they are not equal)
    #[cold]
    #[inline(never)]
    fn incorrect_size_panic(&self, dst_len: usize, src_len: u64) -> ! {
        use core::fmt::Write;

        let mut fmt_buffer = FmtBuffer::<512>::new();
        let _ = write!(
            &mut fmt_buffer,
            "Attempted to copy data from a UserSlice ({} bytes) into an incorrectly sized slice ({} bytes)",
            src_len,
            dst_len
        );

        kernel_panic(
            PanicCode::InternalKernelError,
            fmt_buffer.as_str()
        );
    }
}
