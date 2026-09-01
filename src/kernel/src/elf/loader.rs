// SPDX-License-Identifier: GPL-3.0-only
//! The ELF loader logic
//!
//! Authors: MarioS271

use crate::elf::defs::error::ElfError;
use crate::elf::defs::header::ElfHeader;
use crate::elf::defs::phdrs::ElfPhdr;

/// Validate a given ELF binary slice
/// > **Important**: currently only works for x86_64, will be refactored into a multi-arch friendly layout later
pub fn validate(elf: &[u8]) -> Result<&ElfHeader, ElfError> {
    use ElfError::*;

    if elf.len() < 64 {
        return Err(TooSmall);
    }

    let header = ElfHeader::new(elf);

    for (i, &byte) in ElfHeader::ELF_MAGIC.iter().enumerate() {
        if header.e_ident[i] != byte {
            return Err(BadMagic)
        }
    }

    if header.e_ident[4] != 0x02 {
        return Err(Not64Bit)
    }

    if header.e_ident[5] != 0x01 {
        return Err(NotLittleEndian)
    }

    if header.e_type != ElfHeader::ET_EXEC && header.e_type != ElfHeader::ET_DYN {
        return Err(NotExecutable)
    }

    if header.e_machine != ElfHeader::EM_X86_64 {
        return Err(WrongArch)
    }

    if header.e_phentsize != size_of::<ElfPhdr>() as u16 {
        return Err(BadPhentsize)
    }

    Ok(header)
}
