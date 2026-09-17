// SPDX-License-Identifier: GPL-3.0-only
//! Central kernel state aggregate ([`KState`]): one static holding every OS
//! subsystem's state.
//!
//! Authors: MarioS271

use crate::cpu::state::Cpu;
use crate::kprint::state::KPrint;
use crate::mm::state::Mm;
use crate::sched::state::sched::Sched;
use crate::vfs::state::Vfs;

pub static KSTATE: KState = KState {
    cpu: Cpu::new(),
    kprint: KPrint::new(),
    mm: Mm::new(),
    sched: Sched::new(),
    vfs: Vfs::new()
};

/// The central kernel state aggregate; one field per OS subsystem domain.
pub struct KState {
    pub cpu: Cpu,
    pub kprint: KPrint,
    pub mm: Mm,
    pub sched: Sched,
    pub vfs: Vfs
}

impl KState {
    /// Calls the `init` method of all subsystems which have one
    ///
    /// > This method should be the very first call the kernel makes, as without it,
    /// a lot of [`KSTATE`] will be in an incorrect state for the boot process
    ///
    /// Use this to initialize any non-zero default values; those aren't initialized in
    /// the const fn new, as that would increase the binary size by the size of the entire KSTATE
    /// struct, which we don't want
    pub fn init(&self) {
        self.kprint.init();
        self.mm.init();
        self.sched.init();
    }
}
