// SPDX-License-Identifier: GPL-3.0-only
//! A datatype which describes a region of memory, used when memory only gets mapped in paging early
//! on and later recieves VMAs
//!
//! Authors: MarioS271

use crate::lib::addr::{PhysAddr, VirtAddr};
use crate::lib::types::boot_info::KernelSectionInfo;
use crate::mm::layout;
use crate::mm::vmm::vma::VmaFlags;

/// A data structure to define a memory mapping for when its definition needs to be remembered briefly
/// (like when VMAs and pages get build at different times)
pub struct BootMapping {
    pub virt: VirtAddr,
    pub phys: PhysAddr,
    pub size: u64,
    pub flags: VmaFlags,
    pub can_overmap: bool
}

/// A type to avoid repeating `[BootMapping; 4]` everywhere
pub type BootMappings = [BootMapping; 4];

pub fn build_boot_mappings(sections: &KernelSectionInfo, total_mem: u64) -> BootMappings {
    let image_offset = sections.kernel_phys_start.wrapping_sub(layout::KERNEL_IMAGE_BASE.as_u64());

    [
        // HHDM
        BootMapping {
            virt: layout::KERNEL_HHDM_BASE,
            phys: PhysAddr::new(0),
            size: total_mem,
            flags: VmaFlags::READ | VmaFlags::WRITE,
            can_overmap: true
        },
        // .text
        BootMapping {
            virt: VirtAddr::new(sections.kernel_start),
            phys: PhysAddr::new(sections.kernel_start.wrapping_add(image_offset)),
            size: sections.kernel_text_end - sections.kernel_start,
            flags: VmaFlags::READ | VmaFlags::EXEC,
            can_overmap: false
        },
        // .rodata
        BootMapping {
            virt: VirtAddr::new(sections.kernel_text_end),
            phys: PhysAddr::new(sections.kernel_text_end.wrapping_add(image_offset)),
            size: sections.kernel_rodata_end - sections.kernel_text_end,
            flags: VmaFlags::READ,
            can_overmap: false
        },
        // .data / .bss
        BootMapping {
            virt: VirtAddr::new(sections.kernel_rodata_end),
            phys: PhysAddr::new(sections.kernel_rodata_end.wrapping_add(image_offset)),
            size: sections.kernel_end - sections.kernel_rodata_end,
            flags: VmaFlags::READ | VmaFlags::WRITE,
            can_overmap: false
        }
    ]
}
