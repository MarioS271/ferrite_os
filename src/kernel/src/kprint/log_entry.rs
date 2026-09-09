// SPDX-License-Identifier: GPL-3.0-only
//! Metadata for log entries (Log Entry Header and Log Level definitions)
//!
//! Authors: MarioS271

use crate::lib::macros::bitflags::bitflags;
use crate::lib::types::packed_u8::PackedU8;

// TODO: full POSIX syslog compat

/// Header of one ring buffer log entry, after it comes `text_len` bytes of text
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LogEntryHeader {
    pub seq: u64,
    pub ts_nsec: u64,
    pub text_len: u16,
    /// Upper 5 bit are flags, lower 3 are log level
    pub flags_and_level: PackedU8,
}
impl LogEntryHeader {
    pub const FLAGS_START_BIT: u8 = 0;
    pub const FLAGS_NUM_BITS: u8 = 5;
    pub const LEVEL_START_BIT: u8 = 5;
    pub const LEVEL_NUM_BITS: u8 = 3;
}

/// Struct which represents the kernel log levels
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warn = 4,
    Notice = 5,
    Info = 6,
    Debug = 7
}
impl LogLevel {
    /// Checks if a [`u8`] is a valid log level
    pub const fn is_valid_loglevel(value: u8) -> bool {
        use LogLevel::*;
        if value >= Emergency as u8 && value <= Debug as u8 {
            return true;
        }
        false
    }

    /// Constructs a new [`LogLevel`] from a [`u8`]
    pub const fn from_u8(value: u8) -> Option<Self> {
        use LogLevel::*;
        if !Self::is_valid_loglevel(value) {
            return None;
        }
        Some(match value {
            0 => Emergency,
            1 => Alert,
            2 => Critical,
            3 => Error,
            4 => Warn,
            5 => Notice,
            6 => Info,
            7 => Debug,
            _ => unreachable!()
        })
    }

    /// Returns the starting letter for that log level as a byte
    pub const fn get_letter(&self) -> u8 {
        use LogLevel::*;
        match self {
            Emergency => b'!',
            Alert => b'A',
            Critical => b'C',
            Error => b'E',
            Warn => b'W',
            Notice => b'N',
            Info => b'I',
            Debug => b'D'
        }
    }

    /// Returns a color for that log level in the format of `00 RR GG BB`
    pub const fn get_color(&self) -> u32 {
        use LogLevel::*;
        match self {
            Emergency => 0x00ff00aa,
            Alert => 0x00ffaa00,
            Critical => 0x008a00ff,
            Error => 0x00ff0000,
            Warn => 0x00ffff00,
            Notice => 0x0000ffaa,
            Info => 0x0000ff00,
            Debug => 0x000000ff
        }
    }
}

bitflags!(
    /// Bitflags to further define log entries (does it have a line feed? is it just a part of a full log?)
    LogFlags, u8
);
impl LogFlags {
    pub const NEWLINE: Self = Self(1 << 0);
    pub const CONT: Self = Self(1 << 1);
}
