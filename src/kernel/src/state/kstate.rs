// SPDX-License-Identifier: GPL-3.0-only
//! Central kernel state aggregate ([`KState`]): one static holding every OS
//! subsystem's state.
//!
//! Authors: MarioS271

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
    cpu: Cpu::new(),
    fs: Fs {},
    procs: Procs {},
};

/// The central kernel state aggregate; one field per OS subsystem domain.
pub struct KState {
    pub devs: Devs,
    pub vdevs: VDevs,
    pub sys: Sys,
    pub mnt: Mounts,
    pub net: Net,
    pub ipc: Ipc,
    pub sched: Sched,
    pub mm: Mm,
    pub irq: Irq,
    pub time: Time,
    pub cpu: Cpu,
    pub fs: Fs,
    pub procs: Procs,
}

impl KState {
    /// Return a `'static` reference to the global kernel state.
    pub fn get() -> &'static Self {
        &KSTATE
    }
}
