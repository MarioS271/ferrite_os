//! arch/exceptions/mod.rs
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

//! This module contains exception handlers for all kinds of faults

pub(super) mod double_fault;
pub(super) mod general_protection_fault;
pub(super) mod page_fault;