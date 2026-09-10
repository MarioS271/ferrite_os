// SPDX-License-Identifier: GPL-3.0-only
//! All locations of where things like HHDM, kernel image and more are mapped in the
//! processor's address space
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;

// TODO: ASLR

// Higher Half (Kernel)
pub const KERNEL_IMAGE_BASE: VirtAddr = VirtAddr::new(0xffff_ffff_8000_0000);
pub const KERNEL_IMAGE_SIZE: u64 = 1 << 31;  // 2 GiB

pub const KERNEL_MMIO_BASE: VirtAddr = VirtAddr::new(0xffff_f000_0000_0000);
pub const KERNEL_MMIO_SIZE: u64 = 1 << 43;  // 8 TiB

pub const KERNEL_PAGE_ARRAY_BASE: VirtAddr = VirtAddr::new(0xffff_e000_0000_0000);
pub const KERNEL_PAGE_ARRAY_SIZE: u64 = 1 << 44;  // 16 TiB

pub const KERNEL_VMAP_BASE: VirtAddr = VirtAddr::new(0xffff_d000_0000_0000);
pub const KERNEL_VMAP_SIZE: u64 = 1 << 44;  // 16 TiB

pub const KERNEL_STACKS_BASE: VirtAddr = VirtAddr::new(0xffff_c000_0000_0000);
pub const KERNEL_STACKS_SIZE: u64 = 1 << 44;  // 16 TiB

pub const KERNEL_HHDM_BASE: VirtAddr = VirtAddr::new(0xffff_8000_0000_0000);
pub const KERNEL_HHDM_SIZE: u64 = 1 << 46;  // 64 TiB


// Lower Half (Userspace)
pub const USER_MAX: u64 = 0x0000_8000_0000_0000;
pub const USER_STACK_TOP: VirtAddr = VirtAddr::new(0x0000_7fff_ffff_f000);

pub const USER_MMAP_BASE: VirtAddr = VirtAddr::new(0x0000_7000_0000_0000);

pub const USER_MMAP_MIN: VirtAddr = VirtAddr::new(0x0000_0000_0001_0000); // 64 KiB
