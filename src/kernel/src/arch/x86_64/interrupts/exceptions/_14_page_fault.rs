// SPDX-License-Identifier: GPL-3.0-only
//! Page-fault exception handler (vector 14, `#PF`).
//!
//! Authors: MarioS271

use crate::arch::x86_64::mm::vmm::page_type::PageType;
use crate::lib::addr::VirtAddr;
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::types::fmt_buffer::FmtBuffer;
use crate::mm::layout::USER_MAX;
use crate::mm::pmm::FRAME_SIZE;
use crate::mm::vmm::{Vmm, VmmError};
use crate::mm::vmm::traits::VmmPaging;
use crate::mm::vmm::vma::VmaFlags;
use crate::state::kstate::KSTATE;
use core::fmt::Write;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use crate::arch::x86_64::exception_table::search_exception_table;

/// Handle Page Faults; dispatches to [`handle_kernel_address_pf`] and [`handle_user_address_pf`]
pub extern "x86-interrupt" fn handler(
    mut isf: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    let faulting_virt = VirtAddr::new(Cr2::read_raw());

    let is_ring3 = error_code.contains(PageFaultErrorCode::USER_MODE);
    let is_user_addr = faulting_virt.as_u64() < USER_MAX;

    match (is_ring3, is_user_addr) {
        // From Ring 3, accessing a user address
        (true, true) => {
            handle_user_address_pf(
                false,
                &mut isf,
                error_code,
                faulting_virt
            );
        }

        // From Ring 3, accessing a kernel address
        (true, false) => {
            // TODO: actually end process when thats possible here
            kernel_panic(
                PanicCode::Unknown,
                "Attempted to access kernel address from userspace, EXSEGV not implemented yet"
            );
        }

        // From Ring 0, accessing a user address
        (false, true) => {
            handle_user_address_pf(
                true,
                &mut isf,
                error_code,
                faulting_virt
            );
        }

        // From Ring 0, accessing a kernel address
        (false, false) => {
            handle_kernel_address_pf(
                &isf,
                error_code,
                faulting_virt
            );
        }
    }


}

/// Handle a kernel page fault; maps a new page if VMA is found, otherwise panics
#[inline(always)]
fn handle_kernel_address_pf(
    isf: &InterruptStackFrame,
    error_code: PageFaultErrorCode,
    faulting_virt: VirtAddr
) {
    // Safety: interrupts only get enabled after full mm kernel boot, which guarantees that
    // kernel_addr_space is already initialized
    let addr_space = unsafe { KSTATE.mm.kernel_addr_space().lock() };
    let vma = addr_space.find_vma(faulting_virt).unwrap_or_else(
        || kernel_pf_panic(
            "Could not find VMA to map a page for",
            isf, error_code, faulting_virt
        )
    );

    let flags = vma.flags;
    let page_ptr = addr_space.page_ptr();
    drop(addr_space);

    // Safety: interrupts only get enabled after full mm kernel boot, which guarantees that
    // pmm is already initialized
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };
    let frame = pmm.alloc_frame().unwrap_or_else(
        || kernel_pf_panic(
            "Could not map a new page, out of memory",
            isf, error_code, faulting_virt
        )
    );

    // Safety: page_ptr is the correct virtual address of the kernel PML4 and frame is a valid
    // PMM-allocated memory frame
    unsafe {
        if let Err(error) = Vmm::map_page(
            &mut pmm,
            page_ptr,
            faulting_virt.align_down(FRAME_SIZE),
            frame,
            PageType::Normal,
            Vmm::vma_flags_to_page_flags(flags)
        ) {
            // TODO: check the occurred error type and react accordingly

            kernel_panic(
                PanicCode::OutOfMemory,
                "Could not lazy-map a page for the kernel, out of memory"
            );
        }
    }
}

/// Handle Page Faults which happened when accessing a user address (either from userspace or from the
/// kernel in a syscall)
#[inline(always)]
fn handle_user_address_pf(
    from_ring0: bool,
    isf: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
    faulting_virt: VirtAddr
) {
    let active_pid = KSTATE.sched.active_pid();

    // Safety: we are already past boot stage 2 and no user memory is touched in the closure
    let (vma_flags, page_ptr) = unsafe {
        KSTATE.sched.with_process(active_pid, |proc| {
            (proc.addr_space.find_vma(faulting_virt).map(|vma| vma.flags), proc.addr_space.page_ptr())
        }).unwrap_or_else(
            || kernel_pf_panic(
                "Attempted to fetch process data of non-existent process in page fault handler (this is likely a kernel bug)",
                isf, error_code, faulting_virt
            )
        )
    };

    let Some(vma_flags) = vma_flags else {
        user_pf_cant_map_handler(
            from_ring0,
            isf,
            error_code,
            faulting_virt
        );
        return;
    };

    let mapping_allowed =
        if error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) { false }
        else if error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE) && !vma_flags.contains(VmaFlags::WRITE) { false }
        else if error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH) && !vma_flags.contains(VmaFlags::EXEC) { false }
        else { true };

    if !mapping_allowed {
        user_pf_cant_map_handler(
            from_ring0,
            isf,
            error_code,
            faulting_virt
        );
        return;
    }

    // Safety: interrupts only get enabled after full mm kernel boot, which guarantees that
    // pmm is already initialized
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };
    let frame = pmm.alloc_frame_zeroed().unwrap_or_else(
        // this is a kernel bug because memory was reserved but couldn't be mapped,
        // suggesting the VMM's bookkeeping is broken
        || kernel_pf_panic(
            "Could not lazy-map a page for a user process, out of memory (this is a kernel bug)",
            isf, error_code, faulting_virt
        )
    );

    // Safety: page_ptr is the correct virtual address of the kernel PML4 and frame is a valid
    // PMM-allocated memory frame
    unsafe {
        if let Err(error) = Vmm::map_page(
            &mut pmm,
            page_ptr,
            faulting_virt.align_down(FRAME_SIZE),
            frame,
            PageType::Normal,
            Vmm::vma_flags_to_page_flags(vma_flags)
        ) {
            // TODO: check the occurred error type and react accordingly

            // this is a kernel bug because memory was reserved but couldn't be mapped,
            // suggesting the VMM's bookkeeping is broken
            kernel_panic(
                PanicCode::OutOfMemory,
                "Could not lazy-map a page for a user process, out of memory (this is a kernel bug)"
            );
        }
    }
}

/// Handle the user page fault handler being unable to map
#[cold]
fn user_pf_cant_map_handler(
    from_ring0: bool,
    isf: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
    faulting_virt: VirtAddr
) {
    if from_ring0 {
        match search_exception_table(isf.instruction_pointer.as_u64()) {
            Some(fixup) => {
                // Safety: fixup comes from the exception table which is compile-time valid. It
                // points to directly after the faulting copy instruction, where the regs are
                // already in the expected state
                unsafe {
                    isf.as_mut().update(
                        |frame| frame.instruction_pointer = x86_64::VirtAddr::new(fixup)
                    );
                }
            }
            None => {
                kernel_pf_panic(
                    "Kernel accessed unmapped user memory outside of the exception table",
                    isf, error_code, faulting_virt
                )
            }
        }
    } else {
        // TODO: send EXSEGV
        kernel_pf_panic(
            "Process tried accessing unmapped memory (from userspace); TODO: send EXSEGV",
            isf, error_code, faulting_virt
        )
    }
}

/// Wrapper for [`kernel_panic`] which appends the error code, CR2 address and interrupt stack frame
/// onto the panic message
#[cold]
#[inline(never)]
fn kernel_pf_panic(
    panic_message: &str,
    isf: &InterruptStackFrame,
    error_code: PageFaultErrorCode,
    faulting_virt: VirtAddr
) -> ! {
    let mut fmt_buffer = FmtBuffer::<1024>::new();
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
