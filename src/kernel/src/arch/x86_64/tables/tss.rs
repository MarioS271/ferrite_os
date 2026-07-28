// SPDX-License-Identifier: GPL-3.0-only
//! Task State Segment (TSS) initialization.
//!
//! The TSS holds the Interrupt Stack Table (IST): an array of up to 7 stack
//! pointers the CPU can switch to when delivering certain interrupts. IST entries
//! are used for exceptions that must not rely on the current (possibly corrupted)
//! stack — double fault, NMI, debug, and machine-check handlers each get their own
//! guaranteed-valid stack this way.
//!
//! Authors: MarioS271

use crate::kprint;
use crate::types::aligned_stack::AlignedStack;
use spin::Once;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_STACK_INDEX: usize = 0;
pub const DEBUG_IST_STACK_INDEX: usize = 1;
pub const NMI_IST_STACK_INDEX: usize = 2;
pub const MACHINE_CHECK_IST_STACK_INDEX: usize = 3;

/// Owns the `TaskStateSegment` and the four IST stacks it references.
///
/// Stacks live inside the struct so each per-CPU `Tss` instance carries its own
/// backing storage. The instance must not move after [`Tss::init`] is called —
/// the TSS descriptor in the GDT holds a pointer to it.
pub struct Tss {
    tss: Once<TaskStateSegment>,
    ist1: AlignedStack<8192>,
    ist2: AlignedStack<8192>,
    ist3: AlignedStack<8192>,
    ist4: AlignedStack<8192>,
}

impl Tss {
    /// Create a new, uninitialized `Tss`; call [`Tss::init`] to populate it.
    pub const fn new() -> Self {
        Self {
            tss: Once::new(),
            ist1: AlignedStack { array: [0u8; 8192] },
            ist2: AlignedStack { array: [0u8; 8192] },
            ist3: AlignedStack { array: [0u8; 8192] },
            ist4: AlignedStack { array: [0u8; 8192] },
        }
    }

    /// Build the TSS, wiring IST slots 0–3 to this instance's four dedicated stacks.
    ///
    /// Each stack pointer is set to the top (highest address) of its backing array,
    /// since x86 stacks grow downward.
    pub fn init(&self) {
        // Safe: .add(len) computes a one-past-the-end pointer, which is valid per
        // Rust's pointer rules; the arrays are owned by this instance and will not
        // move or be dropped for the lifetime of the kernel.
        unsafe {
            let ist1_top = VirtAddr::from_ptr(self.ist1.array.as_ptr().add(self.ist1.array.len()));
            let ist2_top = VirtAddr::from_ptr(self.ist2.array.as_ptr().add(self.ist2.array.len()));
            let ist3_top = VirtAddr::from_ptr(self.ist3.array.as_ptr().add(self.ist3.array.len()));
            let ist4_top = VirtAddr::from_ptr(self.ist4.array.as_ptr().add(self.ist4.array.len()));

            self.tss.call_once(|| {
                let mut tss = TaskStateSegment::new();
                tss.interrupt_stack_table[DOUBLE_FAULT_IST_STACK_INDEX] = ist1_top;
                tss.interrupt_stack_table[DEBUG_IST_STACK_INDEX] = ist2_top;
                tss.interrupt_stack_table[NMI_IST_STACK_INDEX] = ist3_top;
                tss.interrupt_stack_table[MACHINE_CHECK_IST_STACK_INDEX] = ist4_top;
                tss
            });
        }

        kprint!("Initialized TSS\n");
    }

    /// Return a reference to the initialized `TaskStateSegment`.
    ///
    /// # Panics
    /// Panics if [`Tss::init`] has not been called yet.
    pub fn get(&self) -> &TaskStateSegment {
        self.tss.get().unwrap()
    }
}
