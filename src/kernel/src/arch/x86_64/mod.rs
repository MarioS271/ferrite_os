// SPDX-License-Identifier: GPL-3.0-only
//! x86_64-specific kernel subsystems: GDT, TSS, IDT, PIC, and interrupt handlers.
//!
//! Authors: MarioS271

pub(crate) mod instructions;
pub(crate) mod tables;
mod interrupts;

/// Initialize the TSS, GDT, IDT, and PIC — everything required before enabling interrupts.
pub(crate) fn init() {
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

    interrupts::pic::init();
}
