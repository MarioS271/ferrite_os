//! arch/x86_64/interrupts/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! This module contains definitions and code for the PIC/APIC (Interrups Controllers)

pub(crate) mod exceptions;
pub(crate) mod irqs;
pub(crate) mod pic;
