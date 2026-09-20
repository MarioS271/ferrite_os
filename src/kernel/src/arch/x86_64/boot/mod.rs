// SPDX-License-Identifier: GPL-3.0-only
//! Arch-Specific Boot Code including linker symbols, early kernel init and limine requests
//!
//! Authors: MarioS271

mod entry;
pub(crate) mod mm;

/// Initialize x86_64 specific stuff
///
/// # Safety
/// The caller must ensure the following:
/// - This function is called exactly once
/// - No SMP/threading is currently active
/// - No jump to userspace has been made yet
pub unsafe fn init(boot_info: &crate::lib::types::boot_info::BootInfo) {
    use crate::state::kstate::KSTATE;

    let cpu = &KSTATE.cpu;
    let gdt_setup_info;

    // Safety:
    // - init_tss is called before init_gdt
    // - We're on the BSP and TSS, GDT and GS Info are all for the BSP CPU
    // The rest is guaranteed by this function's safety contract
    unsafe {
        cpu.bsp_cpu_state().init_tss();
        gdt_setup_info = cpu.bsp_cpu_state().init_gdt();
        cpu.global_cpu_state().init_idt();
        cpu.bsp_cpu_state().init_gs_info();
    }
    cpu.global_cpu_state().set_user_selectors(gdt_setup_info.user_code.0, gdt_setup_info.user_data.0);

    super::interrupts::pic::init();

    mm::mm_init(&boot_info.kernel_sections);

    // Safety: GDT was correctly initialized above, the rest is guaranteed by this function's
    // safety contract
    unsafe {
        super::syscall::init(&gdt_setup_info);
    }
}
