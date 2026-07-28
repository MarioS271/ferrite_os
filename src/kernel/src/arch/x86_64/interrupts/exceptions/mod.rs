// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 CPU exception handlers (vectors 0–30).
//!
//! Authors: MarioS271

pub(crate) mod _0_divide_error;
pub(crate) mod _1_debug;
pub(crate) mod _2_non_maskable_interrupt;
pub(crate) mod _3_breakpoint;
pub(crate) mod _4_overflow;
pub(crate) mod _6_invalid_opcode;
pub(crate) mod _7_device_not_avail;
pub(crate) mod _8_double_fault;
pub(crate) mod _10_invalid_tss;
pub(crate) mod _11_segment_not_present;
pub(crate) mod _12_stack_segment_fault;
pub(crate) mod _13_general_protection_fault;
pub(crate) mod _14_page_fault;
pub(crate) mod _16_x87_floating_point;
pub(crate) mod _17_alignment_check;
pub(crate) mod _18_machine_check;
pub(crate) mod _19_simd_floating_point;
pub(crate) mod _20_virtualization;
pub(crate) mod _29_vmm_communication_exception;
pub(crate) mod _30_security_exception;
pub(crate) mod invalid_fault_handler;
