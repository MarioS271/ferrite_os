//! arch/x86_64/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains definitions for the x86_64 architecture such as the GDT, IDT, TSS
//! or exceptions

pub(crate) mod tss;
pub(crate) mod gdt;
pub(crate) mod idt;
mod exceptions;

pub(crate) fn init() {
    tss::init();
    gdt::init();
    idt::init();
}