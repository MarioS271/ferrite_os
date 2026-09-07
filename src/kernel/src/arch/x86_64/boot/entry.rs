// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 Boot Entrypoint
//!
//! Authors: MarioS271

use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::boot_info::{BootInfo, FramebufferInfo, KernelCmdline, KernelSectionInfo};
use crate::lib::types::fmt_buffer::FmtBuffer;
use crate::state::simple_state::SIMPLE_STATE;
use crate::knotice;
use limine::request::{ExecutableCmdlineRequest, FramebufferRequest, HhdmRequest, MemmapRequest, RsdpRequest};

unsafe extern "C" {
    /// Symbol which is located at the start of the kernel
    pub static __kernel_start: u8;
    /// Symbol which is located at the end of the kernel
    pub static __kernel_end: u8;

    /// Symbol which is located one byte after the end of the `.text` section
    pub static __kernel_text_end: u8;
    /// Symbol which is located one byte after the end of the `.rodata` section
    pub static __kernel_rodata_end: u8;
}

pub static LIMINE_CMDLINE_REQUEST: ExecutableCmdlineRequest = ExecutableCmdlineRequest::new();
pub static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
pub static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
pub static LIMINE_RSDP_REQUEST: RsdpRequest = RsdpRequest::new();
pub static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[unsafe(no_mangle)]
pub extern "C" fn kernel_entry() -> ! {
    crate::state::kstate::KSTATE.init();
    
    serial_init();

    knotice!(
        "Ferrite Kernel {}, on x86_64 booted via limine",
        env!("CARGO_PKG_VERSION"),
    );

    let raw_cmdline = LIMINE_CMDLINE_REQUEST
        .response()
        .map(|r| r.cmdline())
        .unwrap_or("debug");
        // .unwrap_or_else(
        //     || kernel_panic(
        //         PanicCode::InitFailure,
        //         "Limine didn't respond to the cmdline request"
        //     )
        // );

    let mut cmdline = KernelCmdline::default();
    if !raw_cmdline.is_empty() {
        cmdline.parse_in_cmd_line(raw_cmdline);
    }

    let hhdm_offset = LIMINE_HHDM_REQUEST
        .response()
        .map(|r| r.offset)
        .unwrap_or_else(
            || kernel_panic(
                PanicCode::InitFailure,
                "Limine didn't respond to the HHDM offset request"
            )
        );

    let framebuffer = LIMINE_FRAMEBUFFER_REQUEST
        .response()
        .and_then(|r| r.framebuffers().first())
        .map(|fb| FramebufferInfo {
            addr: fb.address() as *mut u32,
            pixels_per_row: fb.pitch / 4,
            width: fb.width,
            height: fb.height,
            bpp: fb.bpp
        })
        .unwrap_or_else(
            || kernel_panic(
                PanicCode::InitFailure,
                "Limine didn't respond to the framebuffer request"
            )
        );

    let rsdp_addr = LIMINE_RSDP_REQUEST
        .response()
        .map(|r| r.address as u64);

    let boot_info = BootInfo {
        cmdline,
        hhdm_offset,
        framebuffer,
        kernel_sections: KernelSectionInfo {
            kernel_start: &raw const __kernel_start as u64,
            kernel_text_end: &raw const __kernel_text_end as u64,
            kernel_rodata_end: &raw const __kernel_rodata_end as u64,
            kernel_end: &raw const __kernel_end as u64
        },
        rsdp_addr
    };

    crate::kernel_main(boot_info);
}

/// Initializes the kernel serial logger
fn serial_init() {
    use crate::drivers::tty::uart::{Uart, UartOps};

    // Safety: we are in a non-SMP/non-threading context
    unsafe {
        SIMPLE_STATE.init_uart(
            Uart::com1()
        );
    };

    // Safety: uart was correctly initialized above
    if let Err(err) = unsafe { SIMPLE_STATE.uart().lock().init() } {
        use core::fmt::Write;

        let mut fmt_buffer = FmtBuffer::<32>::new();
        let _ = write!(&mut fmt_buffer, "Failed to initialize UART COM1 ({})", err.as_str());

        // this panic will result in a silent halt, this is pretty much unavoidable here
        kernel_panic(
            PanicCode::InitFailure,
            fmt_buffer.as_str()
        );
    };
}
