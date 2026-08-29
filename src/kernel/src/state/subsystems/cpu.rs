// SPDX-License-Identifier: GPL-3.0-only
//! Per-CPU state and SMP topology placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::arch::tables;
use alloc::boxed::Box;

/// Per-CPU descriptor tables: one TSS and GDT per CPU, plus the shared IDT.
pub struct Cpu {
    pub idt: tables::idt::Idt,
    pub bsp_cpu_state: CpuState,
    pub ap_cpu_states: Option<Box<[CpuState]>>
}
impl Cpu {
    /// Construct with a default TSS and GDT for every CPU and a fresh IDT.
    pub const fn new() -> Self {
        Self {
            idt: tables::idt::Idt::new(),
            bsp_cpu_state: CpuState::new(),
            ap_cpu_states: None
        }
    }
}

pub struct CpuState {
    pub tss: tables::tss::Tss,
    pub gdt: tables::gdt::Gdt,
}

impl CpuState {
    pub const fn new() -> Self {
        Self {
            tss: tables::tss::Tss::new(),
            gdt: tables::gdt::Gdt::new()
        }
    }
}
