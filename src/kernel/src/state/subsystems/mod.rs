// SPDX-License-Identifier: GPL-3.0-only
//! [`KState`] subcategory structs, one per OS subsystem domain.
//!
//! Authors: MarioS271

pub(crate) mod dev;
pub(crate) mod vdev;
pub(crate) mod sys;
pub(crate) mod mnt;
pub(crate) mod net;
pub(crate) mod ipc;
pub(crate) mod sched;
pub(crate) mod mm;
pub(crate) mod irq;
pub(crate) mod time;
pub(crate) mod cpu;
pub(crate) mod fs;
pub(crate) mod procs;
