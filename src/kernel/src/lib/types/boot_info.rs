// SPDX-License-Identifier: GPL-3.0-only
//! Boot Info Struct (and all necessary types for it), used to call kernel_main
//!
//! Authors: MarioS271

use crate::kprint::log_entry::LogLevel;
use crate::kprint::state::LogTargets;

/// Standardized Boot Info passed to [`kernel_main`](crate::kernel_main)
///
/// This does not contain a mem map entry, each arch is responsible for that as mm_init
/// is per arch anyway
pub struct BootInfo {
    pub hhdm_offset: u64,
    pub rsdp_addr: Option<u64>,
    pub kernel_sections: KernelSectionInfo,
    pub cmdline: KernelCmdline,
    pub framebuffer: FramebufferInfo,
}


///////////////// KERNEL CMD LINE /////////////////

/// Kernel CMD line params
pub struct KernelCmdline {
    pub log_level: LogLevel,
    pub log_targets: LogTargets,
    pub panic_action: PanicAction
}
impl KernelCmdline {
    const KEY_PARSE_BUFFER_LEN: usize = 16;
    const VALUE_PARSE_BUFFER_LEN: usize = 32;

    /// Parse a given CMD line into `self`
    pub fn parse_in_cmd_line(&mut self, cmd_line: &str) {
        let tokens = cmd_line.split_ascii_whitespace();

        for token in tokens {
            let mut parts = token.splitn(2, '=');

            let key_ptr = parts.next().unwrap();
            let value_ptr = parts.next().unwrap_or("");

            let mut key = [0u8; Self::KEY_PARSE_BUFFER_LEN];
            let mut value = [0u8; Self::VALUE_PARSE_BUFFER_LEN];

            let key_len = key_ptr.len().min(Self::KEY_PARSE_BUFFER_LEN);
            key[..key_len].copy_from_slice(&key_ptr.as_bytes()[..key_len]);

            let value_len = value_ptr.len().min(Self::VALUE_PARSE_BUFFER_LEN);
            value[..value_len].copy_from_slice(&value_ptr.as_bytes()[..value_len]);

            key.make_ascii_lowercase();
            value.make_ascii_lowercase();

            let key_as_str_slice = || {
                core::str::from_utf8(&key[..key_len]).unwrap_or("")
            };

            let value_as_str_slice = || {
                core::str::from_utf8(&value[..value_len]).unwrap_or("")
            };
            let value_as_u8 = || {
                value_as_str_slice().parse::<u8>().ok()
            };

            match key_as_str_slice() {
                // Key+Value flags
                "loglevel" => {
                    let Some(value_u8) = value_as_u8() else {
                        match value_as_str_slice() {
                            "emergency" | "emerg" => {
                                self.log_level = LogLevel::Emergency;
                                continue;
                            },
                            "alert" => {
                                self.log_level = LogLevel::Alert;
                                continue;
                            }
                            "critical" | "crit" => {
                                self.log_level = LogLevel::Critical;
                                continue;
                            }
                            "error" => {
                                self.log_level = LogLevel::Error;
                                continue;
                            }
                            "warn" => {
                                self.log_level = LogLevel::Warn;
                                continue;
                            }
                            "notice" => {
                                self.log_level = LogLevel::Notice;
                                continue;
                            }
                            "info" => {
                                self.log_level = LogLevel::Info;
                                continue;
                            }
                            "debug" => {
                                self.log_level = LogLevel::Debug;
                                continue;
                            }
                            _ => { continue; }
                        }
                    };

                    let log_level = LogLevel::from_u8(value_u8);
                    if log_level.is_none() { return; }
                    self.log_level = log_level.unwrap();
                }
                "console" => {
                    let tokens = value_as_str_slice().split(',');
                    for token in tokens {
                         match token {
                             "all" => { self.log_targets |= LogTargets::ALL; }
                             "uart" => { self.log_targets |= LogTargets::UART; }
                             "basic-fb" => { self.log_targets |= LogTargets::BASIC_FB; }
                             _ => {}
                         }
                    }
                }
                "panic" => {
                    match value_as_str_slice() {
                        "halt" => { self.panic_action = PanicAction::Halt; }
                        "reboot" => { self.panic_action = PanicAction::Reboot; }
                        _ => {}
                    }
                }

                // Key only flags
                "quiet" => {
                    self.log_level = LogLevel::Warn;
                }
                "debug" => {
                    self.log_level = LogLevel::Debug;
                }

                _ => {}
            }
        }
    }
}
impl Default for KernelCmdline {
    /// Default kernel CMD line params
    fn default() -> Self {
        Self {
            log_level: crate::config::MAX_LOG_LEVEL,
            log_targets: crate::config::DEFAULT_LOG_TARGETS,
            panic_action: crate::config::DEFAULT_PANIC_ACTION
        }
    }
}


/// What the kernel should do at the end of a panic
///
/// The default value should always be 0 to avoid moving structs which statically
/// initialize this from `.bss` to `.data`
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum PanicAction {
    Halt = 0,
    Reboot = 1
}
impl PanicAction {
    /// Construct a new [`PanicAction`] from a `u8`
    pub fn from_u8(value: u8) -> Option<Self> {
        use PanicAction::*;
        match value {
            0 => Some(Halt),
            1 => Some(Reboot),
            _ => None
        }
    }

    /// Check if the given `u8` corresponds to a valid panic action
    pub fn is_valid_panic_action(value: u8) -> bool {
        use PanicAction::*;
        if value >= Halt as u8 && value <= Reboot as u8 {
            return true;
        }
        false
    }
}


///////////////// FRAMEBUFFER INFO /////////////////

/// Bootloader-provided info for constructing the basic framebuffer
pub struct FramebufferInfo {
    pub addr: *mut u32,
    pub pixels_per_row: u64,
    pub width: u64,
    pub height: u64,
    pub bpp: u16
}

/// Virtual addresses of kernel ELF section boundaries
pub struct KernelSectionInfo {
    pub kernel_phys_start: u64,
    pub kernel_start: u64,
    /// This points to the byte after the last `.text` byte
    pub kernel_text_end: u64,
    /// This points to the byte after the last `.rodata` byte
    pub kernel_rodata_end: u64,
    pub kernel_end: u64
}
