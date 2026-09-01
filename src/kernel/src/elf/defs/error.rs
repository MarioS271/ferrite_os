// SPDX-License-Identifier: GPL-3.0-only
//! Enum which describes ELF loading/parsing errors
//!
//! Authors: MarioS271

#[derive(Debug)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    Not64Bit,
    NotLittleEndian,
    NotExecutable,
    WrongArch,
    BadPhentsize,
    UnsupportedType,
    PhdrsOutOfBounds,
    MisalignedPhdrs,
}