// SPDX-License-Identifier: GPL-3.0-only
//! Per-CPU state and SMP topology placeholder for [`KState`].
//!
//! Authors: MarioS271

use crate::lib::sync::niche_cell::NicheCell;
use alloc::boxed::Box;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::cpu::state::*;

// TODO: add getter for cpu_state(cpu_index)

/// Per-CPU descriptor tables: one TSS and GDT per CPU, plus the shared IDT.
#[repr(align(64))]
pub struct Cpu {
    global_cpu_state: GlobalCpuState,
    bsp_cpu_state: CpuState,
    ap_cpu_states: NicheCell<Box<[CpuState]>>
}

impl Cpu {
    /// Construct with a default TSS and GDT for every CPU and a fresh IDT.
    pub const fn new() -> Self {
        Self {
            global_cpu_state: GlobalCpuState::new(),
            bsp_cpu_state: CpuState::new(),
            ap_cpu_states: NicheCell::new(),
        }
    }

    pub unsafe fn init_ap_cpu_states(&self, value: Box<[CpuState]>) {
        unsafe { self.ap_cpu_states.init(value) };
    }

    /// Getter for `Cpu::global_cpu_state`
    pub fn global_cpu_state(&self) -> &GlobalCpuState {
        &self.global_cpu_state
    }

    /// Getter for `Cpu::bsp_cpu_state`
    pub fn bsp_cpu_state(&self) -> &CpuState {
        &self.bsp_cpu_state
    }

    /// Getter for `Cpu::ap_cpu_states`
    ///
    /// # Safety
    /// The caller must guarantee that at this method's entire execution time,
    /// no mutable references or pointers to this data exist or will exist
    pub unsafe fn ap_cpu_states(&self) -> &Option<Box<[CpuState]>> {
        unsafe { self.ap_cpu_states.get() }
    }
}
