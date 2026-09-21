// SPDX-License-Identifier: GPL-3.0-only
//! Userspace transition Code
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::state::kstate::KSTATE;

/// First transition into userspace; uses a fake stack frame and `iretq` to jump to userspace
/// for the first time
///
/// # Safety
/// The caller must guarantee the following:
/// - The process's address space is fully and correctly mapped
/// - `entry` needs to point to valid executable code in that address space
/// - `stack_top` needs to be valid, mapped and user-accessible
/// - The kernel higher half needs to be mapped in the address space
pub unsafe fn initial_userspace_jump(root_page_phys: u64, entry: u64, stack_top: VirtAddr) -> ! {
    let cs = KSTATE.cpu.global_cpu_state().user_code_selector() as u64;
    let ss = KSTATE.cpu.global_cpu_state().user_data_selector() as u64;

    unsafe {
        core::arch::asm!(
            "mov cr3, {pml4}",
            "push {ss}",
            "push {rsp}",
            "push {rflags}",
            "push {cs}",
            "push {rip}",

            "xor eax, eax",
            "xor ebx, ebx",
            "xor ecx, ecx",
            "xor edx, edx",
            "xor esi, esi",
            "xor edi, edi",
            "xor ebp, ebp",
            "xor r8d, r8d",
            "xor r9d, r9d",
            "xor r10d, r10d",
            "xor r11d, r11d",
            "xor r12d, r12d",
            "xor r13d, r13d",
            "xor r14d, r14d",
            "xor r15d, r15d",

            "iretq",

            pml4 = in(reg) root_page_phys,
            ss = in(reg) ss,
            rsp = in(reg) stack_top.as_u64(),
            rflags = in(reg) 0x202u64,
            cs = in(reg) cs,
            rip = in(reg) entry,
            options(noreturn)
        )
    }
}
