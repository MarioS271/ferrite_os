//! arch/mod.rs
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

//! This module contains definitions for the x86_64 architecture such as the GDT, IDT or TSS

pub(crate) mod tss;
pub(crate) mod gdt;
pub(crate) mod idt;
mod exceptions;
