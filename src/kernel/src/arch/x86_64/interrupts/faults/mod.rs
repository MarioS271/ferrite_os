//! arch/x86_64/interrupts/faults/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains exception handlers for all kinds of faults

pub(crate) mod double_fault;
pub(crate) mod general_protection_fault;
pub(crate) mod page_fault;
