// SPDX-License-Identifier: GPL-3.0-only
//! Kernel print macros ([`kinfo!`](crate::kinfo), [`kdebug`](crate::kdebug), ...)
//!
//! Authors: MarioS271

/// Internal kprint handler function
#[inline(never)]
pub fn __kprint(log_level: super::log_entry::LogLevel, args: core::fmt::Arguments<'_>) {
    use super::log_entry::LogFlags;
    use super::kernel_writer::KernelWriter;
    use crate::state::kstate::KSTATE;

    if KSTATE.kprint.config().max_log_level() >= log_level {
        let mut w = KernelWriter::lock(
            log_level,
            LogFlags::NEWLINE
        );
        let _ = core::fmt::write(&mut w, args);
    }
}


/// Log a line at emergency severity (system is unusable).
#[macro_export]
macro_rules! kemerg {
    ($($arg:tt)*) => {
        $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Emergency, format_args!($($arg)*));
    };
}

/// Log a line at alert severity (action must be taken immediately).
#[macro_export]
macro_rules! kalert {
    ($($arg:tt)*) => {
        $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Alert, format_args!($($arg)*));
    };
}

/// Log a line at critical severity.
#[macro_export]
macro_rules! kcrit {
    ($($arg:tt)*) => {
        $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Critical, format_args!($($arg)*));
    };
}

/// Log a line at error severity.
#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => {
        $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Error, format_args!($($arg)*));
    };
}

/// Log a line at warning severity.
#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => {
        $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Warn, format_args!($($arg)*));
    };
}

/// Log a line at notice severity.
#[macro_export]
macro_rules! knotice {
    ($($arg:tt)*) => {
        $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Notice, format_args!($($arg)*));
    };
}

/// Log a line at info severity.
#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => {
        $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Info, format_args!($($arg)*));
    };
}

/// Log a line at debug severity; compiled out unless the `debug-logging` feature is enabled.
#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => {
        {
            #[cfg(feature = "debug-logging")]
            $crate::kprint::macros::__kprint($crate::kprint::log_entry::LogLevel::Debug, format_args!($($arg)*));
        };
    };
}
