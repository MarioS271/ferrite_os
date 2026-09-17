// SPDX-License-Identifier: GPL-3.0-only
//! Task State Segment (TSS): owns the Interrupt Stack Table and its dedicated
//! stacks for exceptions that need a known-good stack.
//!
//! Authors: MarioS271

use crate::kinfo;
use crate::lib::addr::VirtAddr;
use crate::lib::types::aligned_stack::AlignedStack;
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
    ist1: AlignedStack<8192>,
    ist2: AlignedStack<8192>,
    ist3: AlignedStack<8192>,
    ist4: AlignedStack<8192>,
}

/// Safety: `tss` will only be written to by the owner CPU and will only be written to when
/// the CPU transitions from userspace to kernelspace, where no kernel code could've run before
unsafe impl Sync for Tss {}

impl Tss {
    /// Create a new, uninitialized `Tss`; call [`Tss::init`] to populate it.
    pub const fn new() -> Self {
        Self {
            tss: UnsafeCell::new(TaskStateSegment::new()),
            ist1: AlignedStack::new(),
            ist2: AlignedStack::new(),
            ist3: AlignedStack::new(),
            ist4: AlignedStack::new(),
        }
    }

    /// Build the TSS with RSP0 pointing to 0 and the predefined IST stacks
    ///
    /// # Safety
    /// This method must be called exactly once before [`Tss::tss`] is called to avoid undefined
    /// behavior and multiple mutable references.
    pub unsafe fn init(&'static self) {
        let ist1_top = self.ist1.get_stack_top();
        let ist2_top = self.ist2.get_stack_top();
        let ist3_top = self.ist3.get_stack_top();
        let ist4_top = self.ist4.get_stack_top();

        let tss = unsafe { &mut *self.tss.get() };

        tss.privilege_stack_table[0] = VirtAddr::null().as_x86_64();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_STACK_INDEX] = ist1_top.as_x86_64();
        tss.interrupt_stack_table[DEBUG_IST_STACK_INDEX] = ist2_top.as_x86_64();
        tss.interrupt_stack_table[NMI_IST_STACK_INDEX] = ist3_top.as_x86_64();
        tss.interrupt_stack_table[MACHINE_CHECK_IST_STACK_INDEX] = ist4_top.as_x86_64();

        kinfo!("Initialized TSS");
    }

    /// Getter for `Tss::tss`, returns a reference
    ///
    /// # Safety
    /// The caller must guarantee that there are no mutable references or pointers to `self.tss`
    pub unsafe fn tss(&'static self) -> &'static TaskStateSegment {
        unsafe { &*self.tss.get() }
    }

    /// Setter for TSS.RSP0
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That [`Tss::init`] has already been called
    /// - This method is only called on the CPU which owns this TSS
    /// - That no refs/ptrs from [`Tss::tss`] are held when this is called
    /// - `new_rsp0` is a valid pointer to a valid, mapped kernel stack
    pub unsafe fn set_rsp0(&self, new_rsp0: VirtAddr) {
        unsafe { (*self.tss.get()).privilege_stack_table[0] = new_rsp0.as_x86_64() };
    }
}
