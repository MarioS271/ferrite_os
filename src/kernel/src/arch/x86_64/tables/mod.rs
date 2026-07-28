// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 descriptor tables: TSS, GDT, and IDT.
//!
//! Authors: MarioS271

pub(crate) mod tss;
pub(crate) mod gdt;
pub(crate) mod idt;