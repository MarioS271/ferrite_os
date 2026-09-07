// SPDX-License-Identifier: GPL-3.0-only
//! Kernel Writer, locks the logging state (ringbuffer, basic fb state, ...), collects text,
//! builds log entry, writes it and dispatches the full log message to receivers
//!
//! Authors: MarioS271

use crate::kprint::log_entry::{LogFlags, LogLevel};
use crate::kprint::receivers::KPrintReceiver;
use crate::kprint::state::{KPrintState, LogTargets};
use crate::lib::sync::irq_mutex::IrqMutexGuard;
use crate::state::kstate::KSTATE;
use core::fmt::Write;

/// Size of the intermediary log buffer (2 KiB)
pub const WRITE_BUFFER_SIZE: usize = 1 << 11;

/// Wrapper that holds the acquired [`Ringbuffer`] guard and a buffer in which [`core::fmt::write`]
/// calls get written, which then gets written to the ring buffer
///
/// [`Ringbuffer`]: super::ringbuffer::Ringbuffer
pub struct KernelWriter {
    state: IrqMutexGuard<'static, KPrintState>,
    buffer: [u8; WRITE_BUFFER_SIZE],
    text_len: usize,
    log_level: LogLevel,
    log_flags: LogFlags
}
impl KernelWriter {
    /// Acquire the ringbuffer and return a [`KernelWriter`] holding the guard
    pub fn lock(log_level: LogLevel, log_flags: LogFlags) -> Self {
        Self {
            state: KSTATE.kprint.lock(),
            buffer: [0u8; WRITE_BUFFER_SIZE],
            text_len: 0,
            log_level,
            log_flags
        }
    }
}
impl Write for KernelWriter {
    /// Write `string` into the intermediary write buffer
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        let bytes = string.as_bytes();
        let remaining = WRITE_BUFFER_SIZE - self.text_len;
        let to_copy = bytes.len().min(remaining);

        self.buffer[self.text_len..self.text_len + to_copy]
            .copy_from_slice(&bytes[..to_copy]);

        self.text_len += to_copy;
        Ok(())
    }
}
impl Drop for KernelWriter {
    /// Write intermediary write buffer into ringbuffer as an entry and output to all kprint receivers
    fn drop(&mut self) {
        if self.log_flags.contains(LogFlags::NEWLINE) {
            if self.text_len < WRITE_BUFFER_SIZE {
                self.buffer[self.text_len] = b'\n';
                self.text_len += 1;
            } else {
                self.buffer[WRITE_BUFFER_SIZE - 1] = b'\n';
            }
        }

        let header = self.state.ring_buffer.write_entry(
            self.log_level,
            self.log_flags,
            &self.buffer[..self.text_len]
        );

        // Safety: from_utf8_unchecked is safe here because we built the intermediary buffer from
        // valid string slices
        let text= unsafe {
            core::str::from_utf8_unchecked(&self.buffer[..self.text_len])
        };

        let log_targets = KSTATE.kprint.config().log_targets();

        if log_targets.contains(LogTargets::UART) {
            super::receivers::uart::UartReceiver::receive(
                &mut *self.state,
                &header,
                text
            );
        }
        if log_targets.contains(LogTargets::BASIC_FB) {
            super::receivers::basic_fb::BasicFbReceiver::receive(
                &mut *self.state,
                &header,
                text
            );
        }
    }
}
