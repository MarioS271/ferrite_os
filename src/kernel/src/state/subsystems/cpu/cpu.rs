// SPDX-License-Identifier: GPL-3.0-only
//! Per-CPU state and SMP topology placeholder for [`KState`].
//!
//! Authors: MarioS271

use alloc::boxed::Box;
use core::cell::UnsafeCell;

#[cfg(target_arch = "x86_64")]
use super::x86_64::*;

/// Per-CPU descriptor tables: one TSS and GDT per CPU, plus the shared IDT.
pub struct Cpu {
    global_cpu_state: GlobalCpuState,
    bsp_cpu_state: CpuState,
    ap_cpu_states: UnsafeCell<Option<Box<[CpuState]>>>
}

/// Safety: `ap_cpu_states` will only be written to once before concurrency (SMP, threading).
/// After that, no more mutable references/pointers will exist.
unsafe impl Sync for Cpu {}

impl Cpu {
    /// Construct with a default TSS and GDT for every CPU and a fresh IDT.
    pub const fn new() -> Self {
        Self {
            global_cpu_state: GlobalCpuState::new(),
            bsp_cpu_state: CpuState::new(),
            ap_cpu_states: UnsafeCell::new(None),
        }
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
    /// This method derefs `self.ap_cpu_state.get()`.
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - No mutable references or pointers to `self.ap_cpu_states` must be held
    pub fn ap_cpu_states(&self) -> &Option<Box<[CpuState]>> {
        unsafe { &*self.ap_cpu_states.get() }
    }
}
