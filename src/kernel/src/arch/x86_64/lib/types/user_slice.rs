// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 specific user slice copying code
//!
//! Authors: MarioS271

pub unsafe fn partial_copy_to_slice_asm(slice: &mut [u8], start_addr: u64, length: u64) -> u64 {
    let leftover_len: u64;
    unsafe {
        core::arch::asm!(
            "2:",
            "rep movsb",
            "3:",

            ".pushsection __ex_table, \"a\"",
            ".balign 4",
            ".long 2b - .",
            ".long 3b - .",
            ".popsection",

            inout("rsi") start_addr => _,
            inout("rdi") slice.as_mut_ptr() => _,
            inout("rcx") length => leftover_len,

            options(nostack, preserves_flags)
        );
    }
    leftover_len
}
