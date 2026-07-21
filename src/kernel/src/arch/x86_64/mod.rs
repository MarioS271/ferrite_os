//! arch/x86_64/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains definitions for the x86_64 architecture such as GDT, IDT, TSS, PIC/APIC,
//! interrupts and more

pub(crate) mod instructions;
mod interrupts;
mod tables;

pub(crate) fn init() {
    tables::tss::init();
    tables::gdt::init();
    tables::idt::init();
    interrupts::pic::init();

    instructions::enable_interrupts();
}
