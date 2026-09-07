// SPDX-License-Identifier: GPL-3.0-only
//! ELF validation logic
//!
//! Authors: MarioS271

use super::defs::error::ElfError;
use super::defs::header::ElfHeader;
use super::defs::phdrs::{parse_phdrs, ElfPhdr};
use crate::{kdebug, kerror};

/// Validate a given ELF binary slice
/// Returns a tuple of a slice of [`ElfPhdr`]s and a [`u64`] with the value of `e_entry`
pub fn validate_elf(elf: &[u8]) -> Result<(&[ElfPhdr], u64), ElfError> {
    let header = match validate_header(elf) {
        Ok(h) => h,
        Err(e) => {
            kerror!("Failed to verify ELF header: {:?}", e);
            return Err(e);
        }
    };

    let phdrs = match parse_phdrs(elf, header) {
        Ok(p) => p,
        Err(e) => {
            kerror!("Failed to verify ELF phdrs: {:?}", e);
            return Err(e);
        }
    };

    kdebug!("Validated ELF slice");

    for phdr in phdrs.iter() {
        kdebug!(
            "- phdr type={}, vaddr={:#x}, filesz={:#x}, memsz={:#x}",
            phdr.p_type, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz
        );
    }

    Ok((phdrs, header.e_entry))
}

/// Validate if a byte slice contains a valid ELF header
fn validate_header(elf: &[u8]) -> Result<&ElfHeader, ElfError> {
    use ElfError::*;

    if elf.len() < 64 {
        return Err(TooSmall);
    }

    let header = ElfHeader::new(elf);

    #[cfg(target_arch = "x86_64")] let target_arch = ElfHeader::EM_X86_64;

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

    if header.e_machine != target_arch {
        return Err(WrongArch)
    }

    if header.e_phentsize != size_of::<ElfPhdr>() as u16 {
        return Err(BadPhentsize)
    }

    Ok(header)
}
