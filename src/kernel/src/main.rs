// SPDX-License-Identifier: GPL-3.0-only
//! Kernel entry point and early-boot global state.
//!
//! Authors: MarioS271

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod elf;
mod init;
mod logging;
mod mem;
mod screen;
mod state;
mod types;

mod panic;

use crate::arch::instructions;
use crate::elf::defs::phdrs::{parse_phdrs, ElfPhdr};
use crate::mem::pmm::FRAME_SIZE;
use crate::mem::vmm::address_space::AddressSpace;
use crate::panic::kernel_panic;
use crate::state::kstate::KSTATE;
use crate::state::simple_state::SIMPLE_STATE;
use crate::types::addr::VirtAddr;
use crate::types::panic_codes::PanicCode;
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};
use x86_64::structures::paging::page_table::PageTableEntry;

static LIMINE_FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static LIMINE_MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();
static LIMINE_HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

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

/// Kernel entry point
#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    serial_init();
    basic_fb_init();
    early_kstate_populate();

    kinfo!("Hello, Ferrite!");
    kdebug!("Debug logging is active!");

    arch::init();
    init::mem::mm_init();
    instructions::enable_interrupts();


    // elf loading + userspace execution for stardance goal (this format is temporary, dw)
    // TODO: refactor this properly into KSTATE.procs and so on

    let (phdrs, e_entry) = validate_elf().unwrap_or_else(
        || loop { unsafe { core::arch::asm!("hlt", options(nostack, nomem)); } }
    );
    let mut addr_space = create_new_addr_space();
    map_phdrs_and_copy_elf(&mut addr_space, phdrs, elf::USER_BINARY);

    let stack_top = setup_user_stack(&mut addr_space);
    jump_to_userspace(&addr_space, e_entry, stack_top);

    // code after this comment won't be reached as the kernel jumps to the elf in load_elf

    kinfo!("Kernel ran successfully!");

    // To halt the kernel on finish (temporary)
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem))
        }
    }
}

/// Initializes the kernel serial logger
fn serial_init() {
    use crate::logging::serial::{Serial, SerialPort, _Serial};

    // Safety: we are in a non-SMP/non-threading context
    unsafe { SIMPLE_STATE.init_serial(
        Serial::new(SerialPort::Serial1)
    ) };
}

/// Initializes the Basic Framebuffer
fn basic_fb_init() {
    use crate::screen::basic::framebuffer::BasicFramebuffer;
    use crate::screen::basic::font::Psf2Font;

    if let Some(fb_response) = LIMINE_FRAMEBUFFER_REQUEST.response()
        && let Some(fb) = fb_response.framebuffers().first()
    {
        // Safety: we are in a non-SMP/non-threading context
        unsafe {
            SIMPLE_STATE.init_basic_fb(BasicFramebuffer::new(fb));
            SIMPLE_STATE.init_basic_fb_psf2_font(Psf2Font::init());
        }
    } else {
        kernel_panic(
            PanicCode::InitFailure,
            "Limine did not provide a valid framebuffer"
        );
    }
}

/// Populates KSTATE with early available info such as the hhdm offset
fn early_kstate_populate() {
    if let Some(hhdm_response) = LIMINE_HHDM_REQUEST.response() {
        KSTATE.mm.set_hhdm_offset(hhdm_response.offset);
    } else {
        kernel_panic(
            PanicCode::InitFailure,
            "Limine did not provide a hddm offset"
        );
    }
}


//////////////////// FROM HERE: ELF LOADING ////////////////////

fn validate_elf() -> Option<(&'static [ElfPhdr], u64)> {
    kinfo!("validating elf...");
    let header = match elf::loader::validate(elf::USER_BINARY) {
        Ok(h) => h,
        Err(e) => {
            kwarn!("failed to verify elf header: {:?}", e);
            return None;
        }
    };

    kinfo!("verified elf header, no problems found");
    let phdrs = match parse_phdrs(elf::USER_BINARY, header) {
        Ok(p) => p,
        Err(e) => {
            kwarn!("invalid phdrs: {:?}", e);
            return None;
        }
    };

    kinfo!("phdrs are valid");

    for phdr in phdrs.iter() {
        kinfo!(
            "- phdr type={}, vaddr={:#x}, filesz={:#x}, memsz={:#x}",
            phdr.p_type, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz
        );
    }

    Some((phdrs, header.e_entry))
}


fn create_new_addr_space() -> AddressSpace {
    use mem::pmm::FRAME_SIZE;

    // Safety: the PMM was already initialized in mm_init
    let pml4_frame = unsafe { KSTATE.mm.pmm() }.lock().alloc_frame().unwrap_or_else(
        || kernel_panic(
            PanicCode::OutOfMemory,
            "Could not allocate a PML4 for the to load ELF, out of memory"
        )
    );
    let elf_pml4 = VirtAddr::from_phys(pml4_frame);
    unsafe {
        core::ptr::write_bytes(elf_pml4.as_mut_ptr::<u8>(), 0x00, FRAME_SIZE as usize);
        // TODO: we can't have x86_64 types here
        core::ptr::copy_nonoverlapping(
            KSTATE.mm.kernel_addr_space().lock().page_ptr().as_ptr::<PageTableEntry>().add(256),
            elf_pml4.as_mut_ptr::<PageTableEntry>().add(256),
            256
        );
    }

    AddressSpace::new(elf_pml4)
}

fn map_phdrs_and_copy_elf(addr_space: &mut AddressSpace, phdrs: &[ElfPhdr], elf: &[u8]) {
    use state::kstate::KSTATE;
    use mem::pmm::FRAME_SIZE;
    use mem::vmm::Vmm;
    use mem::vmm::vma::VmaFlags;

    for phdr in phdrs {
        if phdr.p_type != ElfPhdr::PT_LOAD {
            continue;
        }

        let vaddr_start = VirtAddr::new(phdr.p_vaddr).align_down(FRAME_SIZE);
        let vaddr_end = VirtAddr::new(phdr.p_vaddr + phdr.p_memsz).align_up(FRAME_SIZE);
        let size = vaddr_end.as_u64() - vaddr_start.as_u64();

        let mut vma_flags = VmaFlags::USER;
        if phdr.p_flags & ElfPhdr::PF_R != 0 { vma_flags |= VmaFlags::READ; }
        if phdr.p_flags & ElfPhdr::PF_W != 0 { vma_flags |= VmaFlags::WRITE; }
        if phdr.p_flags & ElfPhdr::PF_X != 0 { vma_flags |= VmaFlags::EXEC; }

        // Safety: the PMM was already initialized in mm_init
        let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

        let res = Vmm::map_region(
            &mut pmm,
            addr_space,
            vaddr_start,
            size,
            vma_flags
        );

        drop(pmm);

        if res.is_err() {
            // not unusable here, just wanna see the "!" :D
            kemerg!("Vmm::map_region failed, halting (error code: {:?}", res.err().unwrap());
            loop {
                unsafe {
                    core::arch::asm!("hlt", options(nostack, nomem));
                }
            }
        }

        copy_segment_data(addr_space, phdr, elf);
    }
}

fn copy_segment_data(addr_space: &AddressSpace, phdr: &ElfPhdr, elf: &[u8]) {
    use mem::vmm::{traits::VmmPaging, Vmm};

    let mut copied = 0u64;

    while copied < phdr.p_filesz {
        let dst_vaddr = phdr.p_vaddr + copied;
        let page_base = VirtAddr::new(dst_vaddr).align_down(FRAME_SIZE);
        let offset_in_page = dst_vaddr - page_base.as_u64();

        let phys = Vmm::translate(addr_space.page_ptr(), page_base).unwrap();
        let remaining_space_in_page = FRAME_SIZE - offset_in_page;
        let remaining_to_copy = phdr.p_filesz - copied;
        let chunk = remaining_space_in_page.min(remaining_to_copy);

        unsafe {
            let src = elf.as_ptr().add((phdr.p_offset + copied) as usize);
            core::ptr::copy_nonoverlapping(
                src,
                phys.as_mut_hhdm_ptr::<u8>().add(offset_in_page as usize),
                chunk as usize
            );
        }

        copied += chunk;
    }
}

fn setup_user_stack(addr_space: &mut AddressSpace) -> VirtAddr {
    use mem::pmm::FRAME_SIZE;
    use mem::vmm::Vmm;
    use mem::vmm::vma::VmaFlags;

    let stack_size = FRAME_SIZE * 4;
    let stack_top = VirtAddr::new(0x0000_7fff_ffff_0000);
    let stack_bottom = stack_top - stack_size;

    // Safety: the PMM was already initialized in mm_init
    let mut pmm = unsafe { KSTATE.mm.pmm().lock() };

    // TODO: lazily map this (needs user pf handler which needs this to be in KSTATE.procs properly)
    let res = Vmm::map_region(
        &mut pmm,
        addr_space,
        stack_bottom,
        stack_size,
        VmaFlags::USER | VmaFlags::READ | VmaFlags::WRITE
    );

    drop(pmm);

    // not unusable here, just wanna see the "!" :D
    if res.is_err() {
        kemerg!("Vmm::map_region failed, halting (error code: {:?}", res.err().unwrap());
        loop {
            unsafe {
                core::arch::asm!("hlt", options(nostack, nomem));
            }
        }
    }

    stack_top
}

fn jump_to_userspace(addr_space: &AddressSpace, entry: u64, stack_top: VirtAddr) -> ! {
    let cs = KSTATE.cpu.global_cpu_state().user_code_selector() as u64;
    let ss = KSTATE.cpu.global_cpu_state().user_data_selector() as u64;

    let pml4_phys = addr_space.page_ptr().as_u64() - KSTATE.mm.hhdm_offset();

    unsafe {
        core::arch::asm!(
            "mov cr3, {pml4}",
            "push {ss}",
            "push {rsp}",
            "push {rflags}",
            "push {cs}",
            "push {rip}",
            "iretq",
            pml4 = in(reg) pml4_phys,
            ss = in(reg) ss,
            rsp = in(reg) stack_top.as_u64(),
            rflags = in(reg) 0x202u64,
            cs = in(reg) cs,
            rip = in(reg) entry,
            options(noreturn)
        )
    }
}
