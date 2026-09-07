// SPDX-License-Identifier: GPL-3.0-only
//! kprint State, lives in KSTATE
//!
//! Authors: MarioS271

use super::ringbuffer::Ringbuffer;
use crate::kprint::log_entry::LogLevel;
use crate::lib::sync::irq_mutex::{IrqMutex, IrqMutexGuard};
use core::sync::atomic::{AtomicU8, Ordering};
use crate::lib::macros::bitflags::bitflags;

/// A struct which goes into [`KSTATE`](crate::state::kstate::KSTATE) and contains an [`IrqMutex`]
/// wrapped [`KPrintState`] and the current `max_log_level` and `log_targets` as [`AtomicU8`]s
#[repr(align(64))]
pub struct KPrint {
    lockable: IrqMutex<KPrintState>,
    config: KPrintConfig
}
impl KPrint {
    /// Constructor; creates a new [`KPrint`] instance with zeroed values.
    /// Init needs to be called before using any values in this struct to set the proper default values
    pub const fn new() -> Self {
        Self {
            lockable: IrqMutex::new(KPrintState::new()),
            config: KPrintConfig::new()
        }
    }

    /// Initialize `self.config` to its correct default values
    pub fn init(&self) {
        self.config.init();
    }

    /// Locks and returns an [`IrqMutexGuard`] of [`KPrintState`]
    pub fn lock(&self) -> IrqMutexGuard<KPrintState> {
        self.lockable.lock()
    }

    /// Getter for `self.config`
    pub fn config(&self) -> &KPrintConfig {
        &self.config
    }
}

/// Inner, mutex-protected struct in [`KPrint`];
/// holds the kernel [`Ringbuffer`] aswell as the current [`BasicFbState`]
pub struct KPrintState {
    pub ring_buffer: Ringbuffer,
    pub basic_fb_state: BasicFbState
}
impl KPrintState {
    /// Constructor; creates a new [`KPrintState`] instance, which contains a newly created
    /// [`Ringbuffer`] and a newly created [`BasicFbState`] struct
    pub const fn new() -> Self {
        Self {
            ring_buffer: Ringbuffer::new(),
            basic_fb_state: BasicFbState::new()
        }
    }
}

/// Holds the kernel log configs like max log level and log targets
#[repr(align(64))]
pub struct KPrintConfig {
    max_log_level: AtomicU8,
    log_targets: AtomicU8
}
impl KPrintConfig {
    /// Constructor; zero-initialize all values.
    ///
    /// > **Important:** The zero-initialized values are NOT the correct defaults. [`Self::init`]
    /// > must be called first to load the correct defaults.
    pub const fn new() -> Self {
        Self {
            max_log_level: AtomicU8::new(0),
            log_targets: AtomicU8::new(0)
        }
    }

    /// Initialize the inner values to their proper default values
    pub fn init(&self) {
        self.max_log_level.store(crate::config::MAX_LOG_LEVEL as u8, Ordering::Release);
        self.log_targets.store(crate::config::DEFAULT_LOG_TARGETS.as_u8(), Ordering::Release);
    }

    /// Sets the max log level that will be shown by all `kprint` macro calls
    pub fn set_max_log_level(&self, log_level: LogLevel) {
        self.max_log_level.store(log_level as u8, Ordering::Release);
    }

    /// Sets the log level targets to which `kprint` will dispatch log entries
    pub fn set_log_targets(&self, log_targets: LogTargets) {
        self.log_targets.store(log_targets.as_u8(), Ordering::Release);
    }

    /// Getter for the current max log level
    pub fn max_log_level(&self) -> LogLevel {
        LogLevel::from_u8(self.max_log_level.load(Ordering::Relaxed)).unwrap_or(crate::config::MAX_LOG_LEVEL)
    }

    /// Getter for the current log targets
    pub fn log_targets(&self) -> LogTargets {
        LogTargets::from_u8(self.log_targets.load(Ordering::Acquire))
    }
}

bitflags!(
    /// Where to log kernel logs from the `kprint` subsystem
    LogTargets, u8
);
impl LogTargets {
    pub const ALL: Self = Self(u8::MAX);
    pub const UART: Self = Self(1 << 0);
    pub const BASIC_FB: Self = Self(1 << 1);

    /// Check whether the given LogTargets is a valid log target
    pub fn is_valid_log_target(value: u8) -> bool {
        if value == Self::UART.as_u8()
            || value == Self::BASIC_FB.as_u8()
        { return true; }
        false
    }
}

/// A struct which holds the basic FB state (cursor position)
pub struct BasicFbState {
    pub cursor_x: usize,
    pub cursor_y: usize
}
impl BasicFbState {
    /// Constructor; creates a new [`BasicFbState`] with the cursor at (0, 0)
    pub const fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0
        }
    }
}
