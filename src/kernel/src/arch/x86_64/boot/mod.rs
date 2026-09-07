// SPDX-License-Identifier: GPL-3.0-only
//! Arch-Specific Boot Code including linker symbols, early kernel init and limine requests
//!
//! Authors: MarioS271

mod entry;
pub(crate) mod mm;

/// Initialize x86_64 specific stuff
pub fn init(boot_info: &crate::lib::types::boot_info::BootInfo) {
    use crate::state::kstate::KSTATE;

    let cpu = &KSTATE.cpu;

    let user_code;
    let user_data;

    // Safety:
    // - These two methods are called exactly once here
    // - init_tss is called before init_gdt
    // - No SMP/threading is currently active
    unsafe {
        cpu.bsp_cpu_state().init_tss();
        (user_code, user_data) = cpu.bsp_cpu_state().init_gdt();
        cpu.global_cpu_state().init_idt();
    }
    cpu.global_cpu_state().set_user_selectors(user_code.0, user_data.0);

    super::interrupts::pic::init();

    mm::mm_init(&boot_info.kernel_sections);
}
