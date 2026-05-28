//! arch/x86_64/exceptions/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains exception handlers for all kinds of faults

pub(super) mod double_fault;
pub(super) mod general_protection_fault;
pub(super) mod page_fault;