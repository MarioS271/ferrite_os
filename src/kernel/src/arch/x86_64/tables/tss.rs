// SPDX-License-Identifier: GPL-3.0-only
//! Task State Segment (TSS): owns the Interrupt Stack Table and its dedicated
//! stacks for exceptions that need a known-good stack.
//!
//! Authors: MarioS271

use crate::kinfo;
use crate::types::addr::VirtAddr;
use crate::types::aligned_stack::AlignedStack;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_STACK_INDEX: usize = 0;
pub const DEBUG_IST_STACK_INDEX: usize = 1;
pub const NMI_IST_STACK_INDEX: usize = 2;
pub const MACHINE_CHECK_IST_STACK_INDEX: usize = 3;

/// Owns the `TaskStateSegment` and its four IST stacks; must not move after
/// [`Tss::init`] (the GDT descriptor points to it).
pub struct Tss {
    tss: UnsafeCell<MaybeUninit<TaskStateSegment>>,
    rsp0: AlignedStack<8192>,  // TODO: DO THIS PROPERLY FOR FUCKS SAKE @ME
    ist1: AlignedStack<8192>,
    ist2: AlignedStack<8192>,
    ist3: AlignedStack<8192>,
    ist4: AlignedStack<8192>,
}

// TODO: add unsafe comment
unsafe impl Sync for Tss {}

impl Tss {
    /// Create a new, uninitialized `Tss`; call [`Tss::init`] to populate it.
    pub const fn new() -> Self {
        Self {
            tss: UnsafeCell::new(MaybeUninit::uninit()),
            rsp0: AlignedStack { array: [0u8; 8192] },
            ist1: AlignedStack { array: [0u8; 8192] },
            ist2: AlignedStack { array: [0u8; 8192] },
            ist3: AlignedStack { array: [0u8; 8192] },
            ist4: AlignedStack { array: [0u8; 8192] },
        }
    }

    /// Build the TSS, wiring IST slots 0–3 to this instance's four dedicated stacks.
    pub fn init(&self) {
        // Safety: .add(len) computes a one-past-the-end pointer, which is valid per
        // Rust's pointer rules; the arrays are owned by this instance and will not
        // move or be dropped for the lifetime of the kernel.
        unsafe {
            let rsp0_top = VirtAddr::from_ptr(self.rsp0.array.as_ptr().add(self.rsp0.array.len()));
            let ist1_top = VirtAddr::from_ptr(self.ist1.array.as_ptr().add(self.ist1.array.len()));
            let ist2_top = VirtAddr::from_ptr(self.ist2.array.as_ptr().add(self.ist2.array.len()));
            let ist3_top = VirtAddr::from_ptr(self.ist3.array.as_ptr().add(self.ist3.array.len()));
            let ist4_top = VirtAddr::from_ptr(self.ist4.array.as_ptr().add(self.ist4.array.len()));

            let mut tss = TaskStateSegment::new();
            tss.privilege_stack_table[0] = rsp0_top.as_x86_64();
            tss.interrupt_stack_table[DOUBLE_FAULT_IST_STACK_INDEX] = ist1_top.as_x86_64();
            tss.interrupt_stack_table[DEBUG_IST_STACK_INDEX] = ist2_top.as_x86_64();
            tss.interrupt_stack_table[NMI_IST_STACK_INDEX] = ist3_top.as_x86_64();
            tss.interrupt_stack_table[MACHINE_CHECK_IST_STACK_INDEX] = ist4_top.as_x86_64();

            (*self.tss.get()).write(tss);
        }

        kinfo!("Initialized TSS");
    }

    /// Return a reference to the initialized `TaskStateSegment`.
    ///
    /// # Safety
    /// TODO: insert safety notice here
    ///
    /// # Panics
    /// Panics if [`Tss::init`] has not been called yet.
    pub fn get(&self) -> &TaskStateSegment {
        unsafe { (*self.tss.get()).assume_init_ref() }
    }
}
