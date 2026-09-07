// SPDX-License-Identifier: GPL-3.0-only
//! Kernel Log Ringbuffer
//!
//! Authors: MarioS271

use crate::kprint::log_entry::{LogEntryHeader, LogFlags, LogLevel};
use crate::lib::types::packed_u8::PackedU8;

/// The kernel ringbuffer
pub struct Ringbuffer {
    seq: u64,
    buffer: [u8; Self::SIZE],
    head: usize,
    tail: usize
}

impl Ringbuffer {
    /// Capacity of the in-memory log ring buffer (1 MiB)
    pub const SIZE: usize = 1 << 20;

    /// Constructor; create a new empty [`Ringbuffer`]
    pub const fn new() -> Self {
        Self {
            seq: 0,
            buffer: [0u8; Self::SIZE],
            head: 0,
            tail: 0
        }
    }

    /// Constructs and writes a [`LogEntryHeader`] followed by the given `text` to the ringbuffer
    pub fn write_entry(
        &mut self,
        log_level: LogLevel,
        log_flags: LogFlags,
        text: &[u8]
    ) -> LogEntryHeader {
        use super::log_entry::LogEntryHeader;

        let seq = self.seq;
        self.seq += 1;

        // TODO: get proper timestamp
        let ts_nsec = 0;

        let mut flags_and_level = PackedU8::default();
        flags_and_level.set(
            LogEntryHeader::FLAGS_START_BIT,
            LogEntryHeader::FLAGS_NUM_BITS,
            log_flags.as_u8()
        );
        flags_and_level.set(
            LogEntryHeader::LEVEL_START_BIT,
            LogEntryHeader::LEVEL_NUM_BITS,
            log_level as u8
        );

        let header = LogEntryHeader {
            seq,
            ts_nsec,
            text_len: text.len() as u16,
            flags_and_level
        };
        let header_bytes = unsafe {
            core::slice::from_raw_parts(
                &raw const header as *const u8,
                size_of::<LogEntryHeader>()
            )
        };

        self.write_bytes(header_bytes);
        self.write_bytes(text);

        header
    }

    /// Write raw bytes into the ring buffer
    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.buffer[self.head % Self::SIZE] = *byte;
            self.head = self.head.wrapping_add(1);

            if self.head.wrapping_sub(self.tail) > Self::SIZE {
                self.advance_tail();
            }
        }
    }

    /// Advance `Ringbuffer::tail` to the next entry
    fn advance_tail(&mut self) {
        let header_size = size_of::<LogEntryHeader>();

        let lo = self.buffer[(self.tail + 16) % Self::SIZE];
        let hi = self.buffer[(self.tail + 17) % Self::SIZE];
        let text_len = u16::from_ne_bytes([lo, hi]) as usize;

        self.tail = self.tail.wrapping_add(header_size + text_len);
    }
}
