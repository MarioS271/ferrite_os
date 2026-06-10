# Ferrite OS — Architecture

> A Linux-compatible kernel written in Rust, targeting UEFI systems with the Limine v8.x bootloader. Long-term goal: full Linux x86_64 ABI compatibility sufficient to run unmodified Linux ELF binaries.

---

## Goals

**In scope now:** the kernel only — memory management, process model, syscall layer, VFS, TTY, and enough to run unmodified Linux ELF binaries.

**Tentative / not yet decided:**
- A native shell (fsh) — may just use an existing shell instead
- Wayland compositor, GNOME, Wine integration — long-term aspirations, not committed

**Kernel goals:**
- Full Linux x86_64 ABI compatibility (run unmodified Linux ELF binaries)
- Native namespace-based VFS (`vdev:/`, `dev:/`, `proc:/`, `sys:/`, `file:/`)
- Both classic Unix paths and native namespace paths accepted everywhere
- Rust throughout

---

## Implementation Status

| Subsystem                  | Status      | Doc                       |
|----------------------------|-------------|---------------------------|
| Serial logging             | Done        | —                         |
| Framebuffer + font + text  | Done        | —                         |
| GDT + TSS                  | Done        | —                         |
| IDT + exception handlers   | Done        | —                         |
| PMM (bitmap allocator)     | Done        | [mem/pmm.md](mem/pmm.md)  |
| VMM (page tables, CR3)     | Done        | [mem/vmm.md](mem/vmm.md)  |
| Heap allocator             | Partial     | [mem/heap.md](mem/heap.md)|
| Scheduler                  | Planned     | —                         |
| ELF loader                 | Planned     | —                         |
| Syscall layer              | Planned     | —                         |
| Process model              | Planned     | —                         |
| VFS + namespace layer      | Planned     | —                         |
| TTY + line discipline      | Planned     | —                         |
| Signal layer               | Planned     | —                         |

---

## Multi-Arch Code Organization

`arch/`, `logging/`, `mem/` and so on each contain an `x86_64/` and `aarch64/` subdirectory (and any future ones that may come).
The parent `mod.rs` in each does two things:

1. **Selects** the correct submodule at compile time via `#[cfg(target_arch = "...")]`.
2. **Re-exports** its contents with `pub(crate) use arch_name::*`, so all call sites use the parent path (e.g. `arch::init()`, `mem::vmm::init()`) without knowing which arch is underneath.

This however requires the implementations for all archs to be API-compatible.

`pmm` is shared (not arch-gated) because the bitmap allocator has no architecture-specific code. `heap` is also shared. Only `vmm` is inside the `x86_64/` submodule.

### Required interface per module

| Module     | What each arch submodule must provide                                        |
|------------|------------------------------------------------------------------------------|
| `arch/`    | `pub(crate) fn init()` — sequences hardware init (TSS → GDT → IDT on x86_64) |
| `logging/` | serial write backend consumed by `kprint!`                                   |
| `mem/`     | `pub(crate) mod vmm` — virtual memory manager                                |

### Adding a new architecture

1. Create a subdirectory (e.g. `arch/riscv64/`).
2. Implement the required interface listed above.
3. Add the `#[cfg]` / `pub(crate) use` pair in the parent `mod.rs`.

---

## Actual Boot Sequence

What `kmain()` actually does today:

```
Limine bootloader
    ↓
kmain()
    ├── init COM1 serial
    ├── init framebuffer + font         (from Limine FramebufferRequest)
    ├── arch::init()                    (TSS → GDT → IDT)
    ├── pmm::init()                     (bitmap frame allocator)
    ├── vmm::init()                     (own PML4, switch CR3)
    ├── [VMM test: map/write/remap]     (temporary, will be removed)
    └── hlt loop
```

The rest of the planned boot sequence (heap init, scheduler, VFS, TTY, init process) is not yet implemented.

### Planned boot sequence (future)

```
... (above) ...
    ├── init heap
    ├── init scheduler
    ├── init VFS + namespaces
    ├── init TTY layer
    └── spawn init process (pid 1)
```

---

## Memory Layout

```
0x0000000000000000  ─┐
                     │  Userspace (low half)
0x00007FFFFFFFFFFF  ─┘

── non-canonical gap ──

0xFFFF800000000000  ─┐
                     │  HHDM (direct physical map, Limine-provided)
0xFFFF807FFFFFFFFF  ─┘

0xFFFF808000000000     First safe kernel virtual region (P4 index 257)
                       Heap base: 0xFFFF_8080_0000_0000

0xFFFFFFFF80000000  ─┐
                     │  Kernel image
0xFFFFFFFFFFFFFFFF  ─┘
```

Physical ↔ virtual translation:
```
virt = phys + HHDM_OFFSET
phys = virt - HHDM_OFFSET
```

P4 index 256 (`0xFFFF800000000000`–`0xFFFF807FFFFFFFFF`) is Limine's HHDM with huge pages — do not map into it. First safe kernel virtual region starts at P4 index 257 (`0xFFFF808000000000`).

---

## Module Structure

```
src/kernel/src/
├── main.rs
├── panic.rs                              # kernel_panic() + #[panic_handler]
├── arch/
│   ├── mod.rs                            # cfg-gates + pub(crate) use; no logic
│   ├── x86_64/
│   │   ├── mod.rs                        # declares submodules; init() sequences tss → gdt → idt
│   │   ├── tss.rs                        # Once<TaskStateSegment>; init() sets IST[0] for double fault
│   │   ├── gdt.rs                        # Once<GdtData>; init() loads GDT, reloads CS, loads TSS
│   │   ├── idt.rs                        # init() registers exception handlers, loads IDT
│   │   ├── instructions.rs               # safe wrappers: enable/disable_interrupts(), halt_cpu()
│   │   └── exceptions/
│   │       ├── double_fault.rs
│   │       ├── general_protection_fault.rs
│   │       └── page_fault.rs
│   └── aarch64/
│       └── mod.rs                        # compile_error! stub
├── logging/
│   ├── mod.rs                            # cfg-gates + pub(crate) use
│   ├── kprint.rs                         # kprint! macro + KernelWriter (serial + framebuffer)
│   ├── x86_64/
│   │   ├── mod.rs
│   │   └── serial.rs                     # COM1 serial via 16550 UART port I/O
│   └── aarch64/
│       └── mod.rs                        # compile_error! stub
├── mem/
│   ├── mod.rs                            # cfg-gates (vmm only) + pub(crate) use; pmm and heap unconditional
│   ├── pmm.rs                            # bitmap frame allocator (arch-independent)
│   ├── heap.rs                           # #[global_allocator]; init not yet wired
│   ├── x86_64/
│   │   ├── mod.rs
│   │   ├── vmm.rs                        # VMM: init, get, map_page, unmap_page; Once<VmmData>
│   │   └── vmm_helpers.rs                # alloc_zeroed_frame, out_of_memory_panic, invalid_unmap_panic
│   └── aarch64/
│       └── mod.rs                        # compile_error! stub
├── screen/
│   └── basic/
│       ├── framebuffer.rs                # Once<BasicFramebufferData>, init + get; lock-free
│       ├── font.rs                       # PSF2 font parsing + draw_char; Once<Psf2Header>
│       └── text.rs                       # draw_string, print_to_basic_fb (with scrolling)
└── types/
    └── panic_codes.rs                    # PanicCode enum (u16 repr)
```

---

## Planned Subsystems (Design Specs)

These subsystems are planned but not yet implemented. Their design docs will be added as work begins.

- **Process model** — `Process` struct, fork/exec/wait semantics matching Linux exactly, CoW
- **Syscall layer** — Linux x86_64 ABI via `syscall`/`sysret`, syscall numbers match Linux
- **VFS + namespace layer** — path normalizer, namespace trait, `vdev:/` `dev:/` `proc:/` `sys:/` `file:/`
- **TTY layer** — termios, line discipline, PTY pair, job control signals
- **Signal layer** — `SignalState`, `SigAction`, delivery at syscall exit / interrupt return, `SA_RESTART`
- **Linux binary detection** — `Personality` enum set at `execve`, detected via PT_INTERP / .note.ABI-tag / ELF os_abi

---

## Build Order

```
PMM (bitmap allocator)
    → VMM (page tables, page fault handler, CoW)
        → kernel heap
            → scheduler + context switch
                → ELF loader (static binaries first)
                    → arch_prctl + TLS
                        → fork / exec / exit / wait
                            → signals (basic delivery)
                                → VFS skeleton + namespace router
                                    → vdev:/ (null zero random tty)
                                    → proc:/ (meminfo cpuinfo self/*)
                                    → file:/ (ramfs)
                                        → TTY + line discipline
                                        → PTY pair
                                        → dynamic linker (musl)
                                        → full signals (SA_RESTART, sigaltstack)
                                        → job control (SIGTTOU SIGTTIN SIGTSTP)
                                        → [shell — native or ported, TBD]
                                            → dev:/ (DRM/KMS, input)       [?]
                                            → real filesystem (ext2/fat)    [?]
                                            → networking                    [?]
                                            → Wayland/Weston                [?]
```

---

## Compatibility Targets

Targets up to and including a running shell are committed. Everything after is aspirational.

| Milestone                     | What it proves                          | Status       |
|-------------------------------|-----------------------------------------|--------------|
| musl statically linked binary | PMM + VMM + syscalls + ELF loader work  | planned      |
| busybox sh                    | procfs, signals, TTY line discipline    | planned      |
| bash + coreutils              | Full POSIX process model, job control   | planned      |
| Xorg/twm                      | DRM/KMS driver, input subsystem         | aspirational |
| GTK3 app                      | D-Bus, fontconfig, GIO                  | aspirational |
| XFCE                          | Light desktop, reduced systemd coupling | aspirational |
| GNOME                         | Full stack                              | aspirational |
