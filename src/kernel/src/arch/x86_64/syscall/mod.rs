// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 Syscalls
//!
//! Authors: MarioS271

mod entry;
mod frame;

use crate::arch::x86_64::cpu::tables::gdt::GdtSetupInfo;
use crate::lib::addr::VirtAddr;
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use x86_64::registers::control::{Efer, EferFlags};
use x86_64::registers::model_specific::{LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

/// Enable the `syscall`/`sysret` instructions for this CPU and set the syscall entry point
///
/// # Safety
/// The caller must guarantee that:
/// - This function is called exactly once per CPU
/// - The GDT is already loaded on that CPU
/// - This function is called before entering userspace for the first time on that CPU
pub(crate) unsafe fn init(info: &GdtSetupInfo) {
    unsafe {
        Efer::update(|efer| efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS))
    }

    Star::write(
        info.user_code, info.user_data, info.kernel_code, info.kernel_data
    ).unwrap_or_else(
        |_| kernel_panic(
            PanicCode::InitFailure,
            "Invalid GDT layout for sysret"
        )
    );

    LStar::write(
        VirtAddr::from_ptr(entry::syscall_entry as *const ()).as_x86_64()
    );

    SFMask::write(
        RFlags::INTERRUPT_FLAG
        | RFlags::DIRECTION_FLAG
        | RFlags::TRAP_FLAG
        | RFlags::ALIGNMENT_CHECK
    );
}
