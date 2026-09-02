// SPDX-License-Identifier: GPL-3.0-only
//! Task State Segment (TSS): owns the Interrupt Stack Table and its dedicated
//! stacks for exceptions that need a known-good stack.
//!
//! Authors: MarioS271

use crate::kinfo;
use crate::types::aligned_stack::AlignedStack;
use core::cell::UnsafeCell;
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_STACK_INDEX: usize = 0;
pub const DEBUG_IST_STACK_INDEX: usize = 1;
pub const NMI_IST_STACK_INDEX: usize = 2;
pub const MACHINE_CHECK_IST_STACK_INDEX: usize = 3;

/// Owns the `TaskStateSegment` and its four IST stacks; must not move after
/// [`Tss::init`] (the GDT descriptor points to it).
pub struct Tss {
    tss: UnsafeCell<TaskStateSegment>,
    rsp0: AlignedStack<8192>,  // TODO: DO THIS PROPERLY FOR FUCKS SAKE @ME
    ist1: AlignedStack<8192>,
    ist2: AlignedStack<8192>,
    ist3: AlignedStack<8192>,
    ist4: AlignedStack<8192>,
}

/// Safety: `tss` will only be written to once before concurrency (SMP, threading)
unsafe impl Sync for Tss {}

impl Tss {
    /// Create a new, uninitialized `Tss`; call [`Tss::init`] to populate it.
    pub const fn new() -> Self {
        Self {
            tss: UnsafeCell::new(TaskStateSegment::new()),
            rsp0: AlignedStack::new(),
            ist1: AlignedStack::new(),
            ist2: AlignedStack::new(),
            ist3: AlignedStack::new(),
            ist4: AlignedStack::new(),
        }
    }

    /// Build the TSS with the given IST and RSP stacks
    ///
    /// # Safety
    /// This method must be called exactly once before [`Tss::tss`] is called to avoid undefined
    /// behavior and multiple mutable references.
    pub unsafe fn init(&'static self) {
        let rsp0_top = self.rsp0.get_stack_top();
        let ist1_top = self.ist1.get_stack_top();
        let ist2_top = self.ist2.get_stack_top();
        let ist3_top = self.ist3.get_stack_top();
        let ist4_top = self.ist4.get_stack_top();

        let tss = unsafe { &mut *self.tss.get() };

        tss.privilege_stack_table[0] = rsp0_top.as_x86_64();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_STACK_INDEX] = ist1_top.as_x86_64();
        tss.interrupt_stack_table[DEBUG_IST_STACK_INDEX] = ist2_top.as_x86_64();
        tss.interrupt_stack_table[NMI_IST_STACK_INDEX] = ist3_top.as_x86_64();
        tss.interrupt_stack_table[MACHINE_CHECK_IST_STACK_INDEX] = ist4_top.as_x86_64();

        kinfo!("Initialized TSS");
    }

    /// Getter for `Tss::tss`, returns a reference
    ///
    /// # Safety
    /// This method derefs `self.tss.get()`.
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - There must be no mutable references or pointers to `self.tss`
    pub unsafe fn tss(&'static self) -> &'static TaskStateSegment {
        unsafe { &*self.tss.get() }
    }
}
