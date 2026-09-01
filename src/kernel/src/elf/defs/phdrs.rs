// SPDX-License-Identifier: GPL-3.0-only
//! ELF program header definitions
//!
//! Authors: MarioS271

use super::error::ElfError;
use super::header::ElfHeader;
use core::slice::from_raw_parts;

#[repr(C)]
pub struct ElfPhdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

impl ElfPhdr {
    pub const PT_LOAD: u32 = 1;
    pub const PF_X: u32 = 1;
    pub const PF_W: u32 = 2;
    pub const PF_R: u32 = 4;


}

pub fn parse_phdrs<'a>(elf: &'a [u8], header: &ElfHeader) -> Result<&'a [ElfPhdr], ElfError> {
    let phoff = header.e_phoff as usize;
    let phnum = header.e_phnum as usize;
    let phentsize = header.e_phentsize as usize;

    let table_size = phnum.checked_mul(phentsize).ok_or(ElfError::PhdrsOutOfBounds)?;
    let end = phoff.checked_add(table_size).ok_or(ElfError::PhdrsOutOfBounds)?;

    if end > elf.len() {
        return Err(ElfError::PhdrsOutOfBounds)
    }

    let base = unsafe { elf.as_ptr().add(phoff) };
    if base as usize % align_of::<ElfPhdr>() != 0 {
        return Err(ElfError::MisalignedPhdrs)
    }

    unsafe {
        Ok(from_raw_parts(base as *const ElfPhdr, phnum))
    }
}
