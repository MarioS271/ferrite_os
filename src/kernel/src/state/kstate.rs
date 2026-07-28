//! Central kernel state aggregate (`KState`).
//!
//! `KState` groups every OS subsystem's state into a single static structure so
//! there is one canonical location for all kernel data. Each field is a dedicated
//! subcategory struct (implementing [`KStateSubCategory`]) that will hold the
//! subsystem's data as it is implemented.
//!
//! Access the global instance via [`KState::get`], which returns a `&'static KState`.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use super::subcats::dev::Devs;
use super::subcats::vdev::VDevs;
use super::subcats::sys::Sys;
use super::subcats::mnt::Mounts;
use super::subcats::net::Net;
use super::subcats::ipc::Ipc;
use super::subcats::sched::Sched;
use super::subcats::mm::Mm;
use super::subcats::irq::Irq;
use super::subcats::time::Time;
use super::subcats::cpu::Cpu;
use super::subcats::fs::Fs;
use super::subcats::procs::Procs;

/// The single global kernel state instance. Accessed via [`KState::get`].
static KSTATE: KState = KState {
    devs: Devs {},
    vdevs: VDevs {},
    sys: Sys {},
    mnt: Mounts {},
    net: Net {},
    ipc: Ipc {},
    sched: Sched {},
    mm: Mm {},
    irq: Irq {},
    time: Time {},
    cpu: Cpu {},
    fs: Fs {},
    procs: Procs {},
};

/// The central kernel state aggregate.
///
/// Each field groups a logical OS domain. Fields are `pub` so subsystems can
/// access their slice of state directly through the `KState` reference returned
/// by [`KState::get`].
pub struct KState {
    /// Physical and virtual device registry (real hardware devices).
    pub devs: Devs,
    /// Virtual device registry (software-emulated devices).
    pub vdevs: VDevs,
    /// System-wide configuration and metadata.
    pub sys: Sys,
    /// Filesystem mount table.
    pub mnt: Mounts,
    /// Network subsystem state.
    pub net: Net,
    /// Inter-process communication state.
    pub ipc: Ipc,
    /// Process scheduler state.
    pub sched: Sched,
    /// Memory management subsystem state.
    pub mm: Mm,
    /// IRQ routing and handler table.
    pub irq: Irq,
    /// System time and clock state.
    pub time: Time,
    /// CPU topology and per-CPU data.
    pub cpu: Cpu,
    /// Virtual filesystem state.
    pub fs: Fs,
    /// Process table.
    pub procs: Procs,
}

impl KState {
    /// Return a `'static` reference to the global kernel state.
    pub fn get() -> &'static Self {
        &KSTATE
    }
}
