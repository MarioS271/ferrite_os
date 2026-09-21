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
mod syscall;
mod vfs;

mod config;

use crate::lib::addr::VirtAddr;
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::boot_info::{BootInfo, FramebufferInfo};
use crate::sched::spawn::{SpawnError, spawn_from_elf};
use crate::state::kstate::KSTATE;
use crate::state::simple_state::SIMPLE_STATE;

// Embedded User binary, only temporary
#[repr(align(8))]
struct AlignedBytes<const N: usize> {
    bytes: [u8; N],
}
const USER_BINARY_LEN: usize = include_bytes!("../resources/user-binary").len();
static USER_BINARY_ALIGNED: AlignedBytes<USER_BINARY_LEN> = AlignedBytes {
    bytes: *include_bytes!("../resources/user-binary"),
};
pub static USER_BINARY: &[u8] = &USER_BINARY_ALIGNED.bytes;


/// Abstract kernel entry point, called from the per-arch entry
pub(crate) fn kernel_main(boot_info: BootInfo) -> ! {
    // Stage 1
    KSTATE.mm.set_hhdm_offset(boot_info.hhdm_offset);
    KSTATE.kprint.config().set_max_log_level(boot_info.cmdline.log_level);
    KSTATE.kprint.config().set_log_targets(boot_info.cmdline.log_targets);
    SIMPLE_STATE.set_panic_action(boot_info.cmdline.panic_action);
    
    basic_fb_init(&boot_info.framebuffer);

    kinfo!("Hello, Ferrite!");
    kdebug!("Debug kprint is active!");

    // Stage 2
    // Safety:
    // - This is called exactly once right here
    // - No SMP/threading is currently active
    // - No jump to userspace has been made yet
    unsafe { arch::init(&boot_info) };
    cpu::instructions::enable_interrupts();


    // Safety: we are already past stage 2
    let pid = match unsafe { spawn_from_elf(USER_BINARY, 0) } {
        Ok(p) => p,
        Err(e) => {
            kernel_panic(
                match e {
                    SpawnError::OutOfMemory => PanicCode::OutOfMemory,
                    SpawnError::InvalidBinary => PanicCode::NoWorkingInit,
                    SpawnError::MappingCollision => PanicCode::MemoryMappingCollision
                },
                "Could not spawn the USER_BINARY process"
            )
        }
    };
    KSTATE.sched.set_active_pid(pid);

    unsafe {
        let (page_ptr, kernel_stack, rip, rsp) = KSTATE.sched.with_process(pid, |p| {
            p.status = sched::process::ProcessStatus::Running;
            (p.addr_space.page_ptr(), p.kernel_stack_top, p.regs.rip, p.regs.rsp)
        }).unwrap_or_else(
            || kernel_panic(
                PanicCode::ProcessNotFound,
                "Could not fetch process info from process map"
            )
        );

        KSTATE.cpu.bsp_cpu_state().set_kernel_stack_top(kernel_stack);

        cpu::userspace::initial_userspace_jump(
            page_ptr.as_u64() - KSTATE.mm.hhdm_offset(),
            rip,
            VirtAddr::new(rsp)
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
    use crate::drivers::gpu::basic_fb::font::Psf2Font;
    use crate::drivers::gpu::basic_fb::framebuffer::BasicFramebuffer;

    // Safety: we are in a non-SMP/non-threading context
    unsafe {
        SIMPLE_STATE.init_basic_fb(BasicFramebuffer::new(fb));
        SIMPLE_STATE.init_basic_fb_psf2_font(Psf2Font::init());
    }
}
