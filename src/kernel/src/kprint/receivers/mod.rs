// SPDX-License-Identifier: GPL-3.0-only
//! KPrint Recievers; recieve finished kprint output to display or output
//!
//! Authors: MarioS271

pub(crate) mod uart;
pub(crate) mod basic_fb;

use super::log_entry::LogEntryHeader;
use super::state::KPrintState;

/// A trait which a kprint reciever should implement for a standardized method to receive
/// kernel logs
pub(crate) trait KPrintReceiver {
    /// Recieve and output logs from kprint
    fn receive(
        state: &mut KPrintState,
        header: &LogEntryHeader,
        text: &str
    );
}
