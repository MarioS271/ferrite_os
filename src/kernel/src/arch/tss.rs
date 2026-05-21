//! arch/tss.rs
//! Task State Segment Struct
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::kprint;
use spin::Once;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub static TASK_STATE_SEGMENT: Once<TaskStateSegment> = Once::new();
static mut DOUBLE_FAULT_STACK: [u8; 4096] = [0u8; 4096];

pub fn init() {
    // Safe because we are still single threaded at this point
    unsafe {
        let top_df_stack_addr = VirtAddr::from_ptr(
            DOUBLE_FAULT_STACK.as_ptr().add(DOUBLE_FAULT_STACK.len())
        );

        TASK_STATE_SEGMENT.call_once(|| {
            let mut tss = TaskStateSegment::new();

            tss.interrupt_stack_table[0] = top_df_stack_addr;

            tss
        });
    }

    kprint!("Initialized TASK_STATE_SEGMENT\n");
}
