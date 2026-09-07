// SPDX-License-Identifier: GPL-3.0-only
//! UART KPrint Receiver, receives kprint output to log to serial UART
//!
//! Authors: MarioS271

use super::KPrintReceiver;
use crate::drivers::tty::uart::UartOps;
use crate::kprint::log_entry::{LogEntryHeader, LogLevel};
use crate::kprint::state::KPrintState;
use crate::state::simple_state::SIMPLE_STATE;

pub struct UartReceiver;

impl KPrintReceiver for UartReceiver {
    fn receive(_state: &mut KPrintState, header: &LogEntryHeader, text: &str) {
        if !SIMPLE_STATE.is_uart_initialized() {
            return;
        }

        let log_level = LogLevel::from_u8(header.flags_and_level.get(
            LogEntryHeader::LEVEL_START_BIT,
            LogEntryHeader::LEVEL_NUM_BITS
        ));
        if log_level.is_none() { return; }
        let log_level_letter = log_level.unwrap().get_letter();

        // Safety: was initialized in stage 0
        let mut lock = unsafe { SIMPLE_STATE.uart().lock() };
        let _ = lock.write_byte(log_level_letter);
        let _ = lock.write_byte(b' ');
        let _ = lock.write_string(text);
    }
}
