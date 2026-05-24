# FerriteOS

An x86-64 operating system written from the ground up in Rust.

## Overview

FerriteOS is a bare-metal OS targeting modern UEFI systems. It uses the [Limine bootloader](https://github.com/limine-bootloader/limine) and is built in Rust.
The kernel is compiled for the `x86_64-unknown-none` target inside a Docker container.
The container creates an ISO image which can then be used in QEMU or similar.

## Requirements

- **Docker Desktop** (for kernel compilation)
- **Python 3.11+** (for the custom build tool)
- **QEMU** with x86-64 support (recommended, not mandatory)
- OVMF firmware files (get from [rust-osdev/ovmf-prebuilt](https://github.com/rust-osdev/ovmf-prebuilt), only needed if QEMU will be used)

## Configuration

Create `run/configs/build.toml` before building:

```toml
[extra_paths]
paths = [
    "C:/Program Files/qemu",
    "C:/Program Files/Docker/Docker/resources/bin",
]
```

`build.toml` is required. If no extra paths are needed, the `[extra_paths]` section can be omitted or left empty.

## Building and Running

```
python run/build-*.py build   # compile kernel + create ISO (skips if sources unchanged)
python run/build-*.py run     # launch QEMU with the ISO
python run/build-*.py all     # build + run in one step
python run/build-*.py clean   # delete build/ and target/
```

The build script compiles the kernel and assembles a bootable ISO inside docker, then launches QEMU.
Incremental builds use MD5 hashing to skip docker when nothing has changed.

## Architecture

| Component   | Details                 |
|-------------|-------------------------|
| Target      | `x86_64-unknown-none`   |
| Bootloader  | Limine v8.x (UEFI only) |

The kernel is a single Cargo workspace member (`src/kernel`).
Compilation uses Rust nightly with `build-std` to build `core` and `compiler_builtins` from source.

<br><hr><br>
This project is licensed under the **GNU General Public License v3.0**. See the [LICENSE](LICENSE) file for details.