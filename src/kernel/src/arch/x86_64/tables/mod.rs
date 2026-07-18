//! arch/x86_64/tables/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains definitions for the x86_64 architecture such as GDT, IDT, TSS, PIC/APIC,
//! interrupts and more

pub(crate) mod tss;
pub(crate) mod gdt;
pub(crate) mod idt;