//! arch/x86_64/tables/tss.rs
//! Task State Segment Struct
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::kprint;
use crate::types::aligned_stack::AlignedStack;
use spin::Once;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub static TASK_STATE_SEGMENT: Once<TaskStateSegment> = Once::new();

static mut DOUBLE_FAULT_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };
static mut IST1_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };
static mut IST2_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };
static mut IST3_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };
static mut IST4_STACK: AlignedStack<8192> = AlignedStack{ array: [0u8; 8192] };

pub fn init() {
    // Safe because we are still single threaded at this point
    unsafe {
        let df_stack_addr = VirtAddr::from_ptr(DOUBLE_FAULT_STACK.array.as_ptr().add(DOUBLE_FAULT_STACK.array.len()));
        let ist1_stack_addr = VirtAddr::from_ptr(IST1_STACK.array.as_ptr().add(IST1_STACK.array.len()));
        let ist2_stack_addr = VirtAddr::from_ptr(IST2_STACK.array.as_ptr().add(IST2_STACK.array.len()));
        let ist3_stack_addr = VirtAddr::from_ptr(IST3_STACK.array.as_ptr().add(IST3_STACK.array.len()));
        let ist4_stack_addr = VirtAddr::from_ptr(IST4_STACK.array.as_ptr().add(IST4_STACK.array.len()));

        TASK_STATE_SEGMENT.call_once(|| {
            let mut tss = TaskStateSegment::new();

            tss.interrupt_stack_table[0] = df_stack_addr;
            tss.interrupt_stack_table[1] = ist1_stack_addr;
            tss.interrupt_stack_table[2] = ist2_stack_addr;
            tss.interrupt_stack_table[3] = ist3_stack_addr;
            tss.interrupt_stack_table[4] = ist4_stack_addr;

            tss
        });
    }

    kprint!("Initialized TASK_STATE_SEGMENT\n");
}
