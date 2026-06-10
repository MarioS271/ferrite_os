# FerriteOS

A bare-metal x86-64 operating system written from scratch in Rust, targeting UEFI systems with the [Limine v8.x](https://github.com/limine-bootloader/limine) bootloader.
Long-term goal: full Linux x86_64 ABI compatibility. Run unmodified Linux ELF binaries on a kernel written entirely in Rust.

---

## What's implemented

| Subsystem                                         | Status  |
|---------------------------------------------------|---------|
| Serial (COM1 UART) + framebuffer text output      | Done    |
| GDT, TSS, IDT + exception handlers                | Done    |
| Physical memory manager (bitmap allocator)        | Done    |
| Virtual memory manager (4-level page tables, CR3) | Done    |
| Heap allocator                                    | Partial |
| Scheduler, ELF loader, syscall layer, VFS, TTY    | Planned |

The kernel is a single Cargo workspace member (`src/kernel`), compiled for `x86_64-unknown-none`.
`no-std`, no OS underneath. Serial output goes over COM1; framebuffer rendering uses a PSF2 font drawn pixel-by-pixel from the Limine framebuffer response.

Architecture-specific code lives under `arch/<arch>/`, `logging/<arch>/`, `mem/<arch>/`.
Each parent `mod.rs` selects the correct submodule at compile time via `#[cfg(target_arch)]` and re-exports it,
so the rest of the kernel uses architecture-independent paths like `arch::init()`.

Full subsystem documentation lives in [`docs/`](docs/):
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — goals, boot sequence, memory layout, milestones
- [`docs/mem/`](docs/mem/) — PMM, VMM, heap
- [`docs/arch/x86_64/`](docs/arch/x86_64/) — GDT, TSS, IDT
- [`docs/logging/`](docs/logging/) — serial, `kprint!`
- [`docs/screen/`](docs/screen/basic/) — framebuffer, font, text rendering
- ...

---

## Requirements

- **Docker Desktop** — kernel is compiled inside a Debian + nightly Rust container
- **Python 3.11+** — custom build tool (`run/build-win.py`)
- **QEMU** with x86-64 support
- **OVMF firmware** (`code.fd` + `vars.fd`) in `run/dependencies/ovmf/` — get from [rust-osdev/ovmf-prebuilt](https://github.com/rust-osdev/ovmf-prebuilt)


## Configuration

Create `run/configs/build.toml` before first use:

```toml
[extra_paths]
paths = [
    "C:/Program Files/qemu",
    "C:/Program Files/Docker/Docker/resources/bin",
]
```

Leave this file empty if you do not have any extra path variables to declare.


## Building and Running

```
python run/build-win.py build   # compile kernel + create ISO (skips if sources unchanged)
python run/build-win.py run     # launch QEMU with UEFI firmware
python run/build-win.py all     # build + run
python run/build-win.py clean   # delete build/ and target/
```

Incremental builds use MD5 hashing to skip the Docker step when nothing has changed. There is no `cargo test` — bare-metal target doesn't support it. Validation means: clean build + boots in QEMU.

---

This project is licensed under the **GNU General Public License v3.0** (`GPL-3.0-only`). See the [LICENSE](LICENSE) file for details.
