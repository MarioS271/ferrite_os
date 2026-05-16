//! mod.rs
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

//! This module contains code for early boot initialization, like setting up the basic framebuffer
//! received by limine or setting up the GDT, IDT and more.

pub mod basic_framebuffer;
pub mod panic_framebuffer;