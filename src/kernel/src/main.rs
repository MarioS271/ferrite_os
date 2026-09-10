// SPDX-License-Identifier: GPL-3.0-only
//! Kernel entry point
//!
//! Authors: MarioS271

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![allow(special_module_name)]

extern crate alloc;

mod arch;
mod cpu;
mod drivers;
mod ipc;
mod kprint;
mod lib;
mod mm;
mod net;
mod sched;
mod state;
mod vfs;

mod config;

use crate::lib::types::boot_info::{BootInfo, FramebufferInfo};
use crate::mm::vmm::address_space::AddressSpace;
use crate::sched::loader::load::setup_user_stack;
use crate::sched::loader::validate::validate_elf;
use crate::state::kstate::KSTATE;
use crate::state::simple_state::SIMPLE_STATE;

// Embedded User binary, only temporary
#[repr(align(8))]
struct AlignedBytes<const N: usize> {
    bytes: [u8; N],
}
static USER_BINARY_ALIGNED: AlignedBytes<4656> = AlignedBytes {
    bytes: *include_bytes!("../resources/user-binary"),
};
pub static USER_BINARY: &[u8] = &USER_BINARY_ALIGNED.bytes;


/// Abstract kernel entry point, called from the per-arch entry
pub(crate) fn kernel_main(boot_info: BootInfo) -> ! {
    // Early KSTATE populate
    KSTATE.mm.set_hhdm_offset(boot_info.hhdm_offset);
    KSTATE.kprint.config().set_max_log_level(boot_info.cmdline.log_level);
    KSTATE.kprint.config().set_log_targets(boot_info.cmdline.log_targets);
    SIMPLE_STATE.set_panic_action(boot_info.cmdline.panic_action);
    
    basic_fb_init(&boot_info.framebuffer);

    kinfo!("Hello, Ferrite!");
    kdebug!("Debug kprint is active!");

    arch::init(&boot_info);
    cpu::instructions::enable_interrupts();

    // very hacky elf loading setup right here :D
    // TODO: refactor this properly into KSTATE.procs and so on

    let (phdrs, e_entry) = validate_elf(USER_BINARY).unwrap_or_else(
        |error| {
            kemerg!("Failed to verify ELF binary ({:?}), halting", error);
            cpu::instructions::halt_forever();
        }
    );

    let mut addr_space = AddressSpace::new_user_addr_space();
    sched::loader::load::map_phdrs_and_copy_elf(&mut addr_space, phdrs, USER_BINARY);
    let stack_top = setup_user_stack(&mut addr_space);

    unsafe {
        cpu::userspace::initial_userspace_jump(
            addr_space.page_ptr().as_u64() - KSTATE.mm.hhdm_offset(),
            e_entry,
            stack_top
        );
    }

    #[allow(unreachable_code)]
    {
        kemerg!("Somehow, kernel_main is still running (this means the jump to the user binary did not succeed)");
        cpu::instructions::halt_forever();
    }
}

/// Initializes the Basic Framebuffer
fn basic_fb_init(fb: &FramebufferInfo) {
    use crate::drivers::gpu::basic_fb::framebuffer::BasicFramebuffer;
    use crate::drivers::gpu::basic_fb::font::Psf2Font;

    // Safety: we are in a non-SMP/non-threading context
    unsafe {
        SIMPLE_STATE.init_basic_fb(BasicFramebuffer::new(fb));
        SIMPLE_STATE.init_basic_fb_psf2_font(Psf2Font::init());
    }
}
