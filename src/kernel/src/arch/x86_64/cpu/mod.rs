// SPDX-License-Identifier: GPL-3.0-only
//! Arch-Specific CPU instruction wrappers and tables such as TSS, GDT, IDT
//!
//! Authors: MarioS271

pub(crate) mod tables;
pub(crate) mod gs_info;
pub(crate) mod instructions;
pub(crate) mod state;
pub(crate) mod userspace;
