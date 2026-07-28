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

/// The single kernel TSS. Stored in a [`spin::Once`] because `TaskStateSegment`
/// is not `const`-constructible. The GDT reads this via `get().unwrap()` when
/// building the TSS descriptor, so `tss::init()` must run before `gdt::init()`.
pub static TASK_STATE_SEGMENT: Once<TaskStateSegment> = Once::new();

/// 8 KiB stack for IST slot 0 (double-fault handler).
/// `static mut` because `VirtAddr::from_ptr` needs a raw pointer to its top.
static IST1_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };

/// 8 KiB stack for IST slot 1 (debug handler).
static IST2_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };

/// 8 KiB stack for IST slot 2 (NMI handler).
static IST3_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };

/// 8 KiB stack for IST slot 3 (machine-check handler).
static IST4_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };

/// Initialize the TSS, wiring up IST slots 0–3 to the four dedicated stacks.
///
/// Each stack pointer is set to the array's top (highest address), since x86 stacks
/// grow downward.
pub fn init() {
    // Safe because we are still single threaded at this point
    unsafe {
        let ist1_stack_addr = VirtAddr::from_ptr(IST1_STACK.array.as_ptr().add(IST1_STACK.array.len()));
        let ist2_stack_addr = VirtAddr::from_ptr(IST2_STACK.array.as_ptr().add(IST2_STACK.array.len()));
        let ist3_stack_addr = VirtAddr::from_ptr(IST3_STACK.array.as_ptr().add(IST3_STACK.array.len()));
        let ist4_stack_addr = VirtAddr::from_ptr(IST4_STACK.array.as_ptr().add(IST4_STACK.array.len()));

        TASK_STATE_SEGMENT.call_once(|| {
            let mut tss = TaskStateSegment::new();

            tss.interrupt_stack_table[0] = ist1_stack_addr;
            tss.interrupt_stack_table[1] = ist2_stack_addr;
            tss.interrupt_stack_table[2] = ist3_stack_addr;
            tss.interrupt_stack_table[3] = ist4_stack_addr;

            tss
        });
    }

    kprint!("Initialized TASK_STATE_SEGMENT\n");
}
