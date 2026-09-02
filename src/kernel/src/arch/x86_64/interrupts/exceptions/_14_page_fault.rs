// SPDX-License-Identifier: GPL-3.0-only
//! Page-fault exception handler (vector 14).
//!
//! Authors: MarioS271

use crate::mem::pmm::FRAME_SIZE;
use crate::mem::vmm::traits::VmmPaging;
use crate::mem::vmm::Vmm;
use crate::mem::x86_64::vmm::page_type::PageType;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::types::addr::VirtAddr;
use crate::types::fmt_buffer::FmtBuffer;
use crate::types::panic_codes::PanicCode;
use core::fmt::Write;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

/// Panic with the faulting address (from CR2), the error code, and the interrupt stack frame.
pub extern "x86-interrupt" fn handler(
    isf: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    let faulting_virt = VirtAddr::new(Cr2::read_raw());

    if error_code.contains(PageFaultErrorCode::USER_MODE) {
        handle_user_pf(faulting_virt);
    } else {
        handle_kernel_pf(isf, error_code, faulting_virt);
    }
}

/// Handle a kernel page fault; maps a new page if VMA is found, otherwise panics
#[inline]
fn handle_kernel_pf(
    isf: InterruptStackFrame,
    error_code: PageFaultErrorCode,
    faulting_virt: VirtAddr
) {
    // Safety: interrupts only get enabled after full mm kernel init, which guarantees that
    // kernel_addr_space is already initialized
    let addr_space = unsafe { KSTATE.mm.kernel_addr_space().lock() };
    let vma = addr_space.find_vma(faulting_virt).unwrap_or_else(
        || kernel_pf_panic(
            "Could not find VMA to map a page for",
            &isf, error_code, faulting_virt
        )
    );

    let flags = vma.flags;
    let page_ptr = addr_space.page_ptr();
    drop(addr_space);

    // Safety: interrupts only get enabled after full mm kernel init, which guarantees that
    // pmm is already initialized
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };
    let frame = pmm.alloc_frame().unwrap_or_else(
        || kernel_pf_panic(
            "Could not map a new page, out of memory",
            &isf, error_code, faulting_virt
        )
    );

    // Safety: page_ptr is the correct virtual address of the kernel PML4 and frame is a valid
    // PMM-allocated memory frame
    unsafe {
        Vmm::map_page(
            &mut pmm,
            page_ptr,
            faulting_virt.align_down(FRAME_SIZE),
            frame,
            PageType::Normal,
            Vmm::vma_flags_to_page_flags(flags)
        );
    }
}

#[inline]
fn handle_user_pf(
    faulting_virt: VirtAddr
) {
    todo!();
}

#[inline]
fn kernel_pf_panic(
    panic_message: &str,
    isf: &InterruptStackFrame,
    error_code: PageFaultErrorCode,
    faulting_virt: VirtAddr
) -> ! {
    let mut fmt_buffer = FmtBuffer::<512>::new();
    let _ = write!(
        &mut fmt_buffer,
        "{}\n\nCR2: {:x}\nError Code: {:?}\n\n{:#?}",
        panic_message, faulting_virt, error_code, isf
    );
    kernel_panic(
        PanicCode::PageFault,
        fmt_buffer.as_str(),
    );
}
