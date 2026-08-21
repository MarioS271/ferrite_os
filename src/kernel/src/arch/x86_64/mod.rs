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

    cpu.tss[0].init();
    cpu.gdt[0].init(&cpu.tss[0]);
    cpu.idt.init();

    interrupts::pic::init();
}
