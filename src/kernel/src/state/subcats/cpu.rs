// SPDX-License-Identifier: GPL-3.0-only
//! Per-CPU state and SMP topology placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::config;
use crate::arch::tables;

/// Holds per-CPU registers, local APIC state, and SMP topology.
///
/// Not yet implemented; currently a placeholder in [`KState`].
pub struct Cpu {
    pub tss: [tables::tss::Tss; config::MAX_CPUS],
    pub gdt: [tables::gdt::Gdt; config::MAX_CPUS],
    pub idt: tables::idt::Idt,
}
impl Cpu {
    pub const fn new() -> Self {
        Self {
            tss: [const { tables::tss::Tss::new() }; config::MAX_CPUS],
            gdt: [const { tables::gdt::Gdt::new() }; config::MAX_CPUS],
            idt: tables::idt::Idt::new(),
        }
    }
}