// SPDX-License-Identifier: GPL-3.0-only
//! Compile-Time Kernel Config; configure the compiled kernel's default params
//!
//! Authors: MarioS271

use crate::kprint::log_entry::LogLevel;
use crate::kprint::state::LogTargets;
use crate::lib::types::boot_info::PanicAction;

////////////////////// LOGGING ////////////////////// 
/// The max log level to use unless configured otherwise by the kernel cmd line
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Info;

/// The default log targets which should recieve logs
pub const DEFAULT_LOG_TARGETS: LogTargets = LogTargets::ALL;


////////////////////// OTHER //////////////////////
/// The default action to take on a kernel panic
///
/// This value should be zero in the [`PanicAction`] definition to avoid moving
/// structs which statically initialize this from `.bss` to `.data`; this is checked by the
/// const assert after this declaration
pub const DEFAULT_PANIC_ACTION: PanicAction = PanicAction::Halt;
const _: () = assert!(
    DEFAULT_PANIC_ACTION as u8 == 0,
    "DEFAULT_PANIC_ACTION is non-zero which would move container structs from .bss to .data, which in turn bloats the kernel binary"
);
