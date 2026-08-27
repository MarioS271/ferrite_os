// SPDX-License-Identifier: GPL-3.0-only
//!
//!
//! Authors: MarioS271

use core::fmt::{Display, Formatter};
use crate::mem::pmm::FRAME_SIZE;

pub const HUGE_PAGE_SIZE_2MIB: u64 = FRAME_SIZE * 512;
pub const HUGE_PAGE_SIZE_1GIB: u64 = HUGE_PAGE_SIZE_2MIB * 512;

#[repr(u64)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PageType {
    Normal = FRAME_SIZE,
    HugePage2MiB = HUGE_PAGE_SIZE_2MIB,
    HugePage1GiB = HUGE_PAGE_SIZE_1GIB
}
impl Display for PageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", match self {
            PageType::Normal => "Normal(4KiB)",
            PageType::HugePage2MiB => "HugePage(2MiB)",
            PageType::HugePage1GiB => "HugePage(1GiB)",
        })
    }
}