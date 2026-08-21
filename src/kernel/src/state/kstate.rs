// SPDX-License-Identifier: GPL-3.0-only
//! Central kernel state aggregate ([`KState`]): one static holding every OS
//! subsystem's state.
//!
//! Authors: MarioS271

use super::subsystems::dev::Devs;
use super::subsystems::vdev::VDevs;
use super::subsystems::sys::Sys;
use super::subsystems::mnt::Mounts;
use super::subsystems::net::Net;
use super::subsystems::ipc::Ipc;
use super::subsystems::sched::Sched;
use super::subsystems::mm::Mm;
use super::subsystems::irq::Irq;
use super::subsystems::time::Time;
use super::subsystems::cpu::Cpu;
use super::subsystems::fs::Fs;
use super::subsystems::procs::Procs;

pub static KSTATE: KState = KState {
    devs: Devs {},
    vdevs: VDevs {},
    sys: Sys {},
    mnt: Mounts {},
    net: Net {},
    ipc: Ipc {},
    sched: Sched {},
    mm: Mm::new(),
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
