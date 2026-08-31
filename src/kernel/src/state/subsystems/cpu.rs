// SPDX-License-Identifier: GPL-3.0-only
//! Per-CPU state and SMP topology placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::arch::tables;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicU16, Ordering};

/// Per-CPU descriptor tables: one TSS and GDT per CPU, plus the shared IDT.
pub struct Cpu {
    pub idt: tables::idt::Idt,
    pub bsp_cpu_state: CpuState,
    pub ap_cpu_states: Option<Box<[CpuState]>>,
    user_code_selector: AtomicU16,
    user_data_selector: AtomicU16,
}
impl Cpu {
    /// Construct with a default TSS and GDT for every CPU and a fresh IDT.
    pub const fn new() -> Self {
        Self {
            idt: tables::idt::Idt::new(),
            bsp_cpu_state: CpuState::new(),
            ap_cpu_states: None,
            user_data_selector: AtomicU16::new(0),
            user_code_selector: AtomicU16::new(0)
        }
    }

    /// Set the user data and code selectors
    pub fn set_user_selectors(&self, code: u16, data: u16) {
        self.user_code_selector.store(code, Ordering::Release);
        self.user_data_selector.store(data, Ordering::Release);
    }

    /// Getter for `KSTATE::cpu::user_code_selector`
    pub fn user_code_selector(&self) -> u16 {
        self.user_code_selector.load(Ordering::Acquire)
    }

    /// Getter for `KSTATE::cpu::user_data_selector`
    pub fn user_data_selector(&self) -> u16 {
        self.user_data_selector.load(Ordering::Acquire)
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
