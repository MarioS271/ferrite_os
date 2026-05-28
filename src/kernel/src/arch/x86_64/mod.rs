//! arch/x86_64/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains definitions for the x86_64 architecture such as the GDT, IDT, TSS
//! or exceptions

mod tss;
mod gdt;
mod idt;
mod exceptions;
pub(crate) mod instructions;

pub(crate) fn init() {
    tss::init();
    gdt::init();
    idt::init();
}