//! arch/x86_64/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains definitions for the x86_64 architecture such as the GDT, IDT, TSS
//! or exceptions

mod tss;
mod gdt;
mod idt;
pub(crate) mod instructions;
mod exceptions;
mod interrupts;

pub(crate) fn init() {
    tss::init();
    gdt::init();
    idt::init();
    interrupts::pic::init();
}
