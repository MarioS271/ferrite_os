// SPDX-License-Identifier: GPL-3.0-only
//! ELF header definitions
//!
//! Authors: MarioS271

#[repr(C)]
pub struct ElfHeader {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl ElfHeader {
    pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
    pub const ET_EXEC: u16 = 2;
    pub const ET_DYN: u16 = 3;
    pub const EM_X86_64: u16 = 0x3E;

    /// Takes an ELF byte slice and returns it as a reference to an [`ElfHeader`]
    ///
    /// # Safety
    /// The caller should make sure that the given byte slice is ELF data to avoid undefined behavior
    pub fn new(elf: &[u8]) -> &Self {
        unsafe { &*(elf.as_ptr() as *const Self) }
    }
}
