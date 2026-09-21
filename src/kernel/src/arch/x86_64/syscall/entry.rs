// SPDX-License-Identifier: GPL-3.0-only
//! Syscall entry handler
//!
//! Authors: MarioS271

use crate::arch::x86_64::cpu::gs_info::{GS_INFO_KERNEL_STACK_TOP, GS_INFO_USER_RSP};
use crate::arch::x86_64::syscall::frame::UserFrame;
use crate::mm::layout::USER_MAX;
use crate::state::kstate::KSTATE;
use crate::syscall::structs::SyscallFrame;
use core::arch::naked_asm;

/// Syscall entry point
///
/// # Safety
/// This function must never be called from rust, it should only ever be reached via the `syscall`
/// instruction. Additionally, it requires that [`GsInfo::init`] and [`GsInfo::set_kernel_stack_top`]
/// have already been called on this CPU
///
/// [`GsInfo::init`]: crate::arch::x86_64::cpu::gs_info::GsInfo::init
/// [`GsInfo::set_kernel_stack_top`]: crate::arch::x86_64::cpu::gs_info::GsInfo::set_kernel_stack_top
#[unsafe(naked)]
pub unsafe extern "C" fn syscall_entry() -> ! {
    naked_asm!(
        "swapgs",
        "mov gs:[{user_rsp}], rsp",
        "mov rsp, gs:[{kernel_stack_top}]",

        "sub rsp, 8",           // _padding
        "push 0",               // ss
        "push 0",               // rsp (placeholder)
        "push r11",             // user rflags
        "mov r11, gs:[{user_rsp}]",
        "mov [rsp + 8], r11",   // rsp
        "push 0",               // cs
        "push rcx",             // user rip
        "push rax",             // orig_rax
        "push rax",             // rax
        "push rdi",             // rdi
        "push rsi",             // rsi
        "push rdx",             // rdx
        "push r8",              // r8
        "push r9",              // r9
        "push r10",             // r10
        "push rbx",             // rbx
        "push rbp",             // rbp
        "push r12",             // r12
        "push r13",             // r13
        "push r14",             // r14
        "push r15",             // r15

        "sti",
        "mov rdi, rsp",
        "call {dispatcher}",

        "cli",
        "pop r15",              // r15
        "pop r14",              // r14
        "pop r13",              // r13
        "pop r12",              // r12
        "pop rbp",              // rbp
        "pop rbx",              // rbx
        "pop r10",              // r10
        "pop r9",               // r9
        "pop r8",               // r8
        "pop rdx",              // rdx
        "pop rsi",              // rsi
        "pop rdi",              // rdi
        "pop rax",              // rax
        "add rsp, 8",           // orig_rax
        "pop rcx",              // rip
        "add rsp, 8",           // cs
        "mov r11, {user_max}",
        "cmp rcx, r11",
        "jae 1f",
        "pop r11",              // rflags
        "pop rsp",              // user rsp

        "swapgs",
        "sysretq",

        "1:",
        "sub rsp, 16",
        "xor r11d, r11d",
        "swapgs",
        "iretq",

        kernel_stack_top = const GS_INFO_KERNEL_STACK_TOP,
        user_rsp = const GS_INFO_USER_RSP,
        dispatcher = sym syscall_dispatch,
        user_max = const USER_MAX
    )
}

/// Dispatches incoming syscalls to the correct handler
extern "C" fn syscall_dispatch(frame: &mut UserFrame) {
    let syscall_frame = SyscallFrame {
        syscall_num: frame.orig_rax,
        arg1: frame.rdi,
        arg2: frame.rsi,
        arg3: frame.rdx,
        arg4: frame.r10,
        arg5: frame.r9,
        arg6: frame.r8,
    };

    let res = crate::syscall::syscall::Syscall::dispatch(&syscall_frame);
    frame.rax = match res {
        Ok(_) => 0u64,
        Err(e) => e as u64
    };
    frame.rdx = res.unwrap_or(0u64);

    frame.cs = KSTATE.cpu.global_cpu_state().user_code_selector() as u64;
    frame.ss = KSTATE.cpu.global_cpu_state().user_data_selector() as u64;
}
