// SPDX-License-Identifier: GPL-3.0-only
//! [`KState`] subcategory structs, one per OS subsystem domain.
//!
//! Authors: MarioS271

pub mod dev;
pub mod vdev;
pub mod sys;
pub mod mnt;
pub mod net;
pub mod ipc;
pub mod sched;
pub mod mm;
pub mod irq;
pub mod time;
pub mod cpu;
pub mod fs;
pub mod procs;
