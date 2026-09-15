// SPDX-License-Identifier: GPL-3.0-only
//! Spawn new processes
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use crate::mm::MmError;
use crate::mm::vmm::address_space::AddressSpace;
use crate::sched::loader::defs::error::ElfError;
use crate::sched::loader::load::{map_phdrs_and_copy_elf, setup_user_stack};
use crate::sched::loader::validate::validate_elf;
use crate::sched::process::{KernelStack, Pid, Process};
use crate::sched::state::sched::Sched;
use crate::state::kstate::KSTATE;

/// Wrapper type for `Result<T, SpawnError>`
pub type SpawnResult<T> = Result<T, SpawnError>;

/// Indicates an error when attempting to spawn a new process
pub enum SpawnError {
    InvalidBinary,
    OutOfMemory,
    MappingCollision
}
impl From<MmError> for SpawnError {
    /// Convert a given [`MmError`] into a [`SpawnError`]
    fn from(value: MmError) -> Self {
        match value {
            MmError::MisalignedAddress | MmError::NotFound => Self::InvalidBinary,
            MmError::OutOfMemory => Self::OutOfMemory,
            MmError::Overlap => Self::MappingCollision
        }
    }
}
impl From<ElfError> for SpawnError {
    /// Convert a given [`ElfError`] into a [`SpawnError`]
    fn from(_: ElfError) -> Self {
        Self::InvalidBinary
    }
}

/// Spawn a new process, returning its PID
///
/// # Safety
/// The caller must guarantee that:
/// - This method is only called post-stage-2 when memory management has already been initialized
/// - [`Sched::init_procs`] has already been called
pub unsafe fn spawn_from_elf(elf: &[u8], parent_pid: Pid) -> SpawnResult<Pid> {
    let (phdrs, e_entry) = validate_elf(elf)?;

    let mut addr_space = unsafe { AddressSpace::new_user_addr_space() };
    map_phdrs_and_copy_elf(&mut addr_space, phdrs, elf)?;

    let user_stack_top = setup_user_stack(&mut addr_space)?;
    let kernel_stack_top = KernelStack { base: VirtAddr::null(), size: 0 };

    let pid = KSTATE.sched.alloc_pid();

    let mut process = Process::new(
        pid,
        parent_pid,
        addr_space,
        kernel_stack_top
    );
    process.regs.rip = e_entry;
    process.regs.rsp = user_stack_top.as_u64();

    unsafe { KSTATE.sched.insert_process(process) };

    Ok(pid)
}
