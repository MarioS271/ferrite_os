# <img src="logo/logo_full/ferrite_logo_full_1000.png" width=300>

A kernel written from scratch in Rust, with the goal of full POSIX-compatibility.
Ferrite aims to be a modern and optimized kernel as well as provide some advanced capabilities
such as intent-aware scheduling, real-time scheduling/scheduling hints ("don't preempt me for this long"),
a namespaced VFS and more. Furthermore, Ferrite aims towards a clean design, fixing a lot of design mistakes in other kernels.

The kernel source tree lives under [src/kernel/src](src/kernel/src) as a cargo workspace member.
Architecture-specific code is isolated under [arch](src/kernel/src/arch) and re-exported into
the right submodule at compile time via `#[cfg(target_arch)]`.

---

## 1. Ferrite's Architecture
| Feature                 | Design Choice(s)                                |
|-------------------------|-------------------------------------------------|
| Language                | Rust                                            |
| Kernel Type             | Monolithic                                      |
| Supported Architectures | x86_64 (more planned)                           |
| Syscall ABI             | POSIX                                           |

### 1.1. The VFS
Ferrite follows the "everything is a file" mindset, with one deviation: it uses a namespaced VFS.
The namespacing aims to separate actual files from things like physical and virtual devices, process info and more.

Namespaces that Ferrite exposes include:

| Namespace | Description                                                                       | Example Path             | Linux Equivalent for Ferrite Example Path |
|-----------|-----------------------------------------------------------------------------------|--------------------------|-------------------------------------------|
| `fs:/`    | The file namespace, contains all real files and mounted disks/partitions          | `fs:/home/user/`         | `/home/user`                              |
| `dev:/`   | Physical Devices (Disks, ...)                                                     | `dev:/sda`               | `/dev/sda`                                |
| `vdev:/`  | Virtual Devices (TTYs, ...)                                                       | `vdev:/tty0`             | `/dev/tty0`                               |
| `hw:/`    | Hardware Info                                                                     | `hw:/cpu/temp`           | `/sys/class/hwmon/hwmon0/temp1_input`     |
| `net:/`   | Network Adapters, Firewall Info, ...                                              | `net:/eth0/stats`        | `/sys/class/net/eth0/statistics`          |
| `ipc:/`   | Inter-Process Communication (Pipes, Sockets, ...)                                 | `ipc:/socket/nginx.sock` | `/run/nginx/nginx.sock`                   |
| `proc:/`  | Process Info                                                                      | `proc:/self`             | `/proc/self`                              |
| `ctl:/`   | Runtime Machine and Kernel configuration (power states, ...)                      | `ctl:/power/state`       | `/sys/power/state`                        |
| `reg:/`   | Ferrite Registry, Central System and App configs (similar to the Windows Registry | `reg:/app1/theme`        | `/etc/*`                                  |
| `log:/`   | Central Logging System; userspace processes can register into here too            | `log:/kernel/`           | `/dev/kmsg`                               |


### 1.2. Memory Management
Ferrite uses a **Buddy Allocator** for physical memory management.
It uses its own VMAs parallel to paging to track memory regions and their access flags.
<br>
Currently, it uses a linked list heap allocator, but this is planned to be rewritten.

The entire physical memory is mapped into the higher half of the address space via the limine-provided **hhdm offset**.
The kernel is mapped into the higher half at `0xffff_ffff_8000_0000`.

### 1.3. The Scheduler
... is not implemented yet! :)

---

## 2. Building, Running and Documentation

### 2.1. Ferrite's build system

Ferrite uses a python build script system, where per arch there is one `build.py`, which can build and immediately also run
the kernel in QEMU. Helpers shared by all scripts (config loading, container lifecycle, command dispatch) live in
[scripts/lib.py](scripts/lib.py). For required software for building/running the kernel, refer to [§ 2.2. Requirements](#22-requirements)

For information about the docker container in which the kernel gets build, refer to [Dockerfile](Dockerfile)
and [docker-compose.yml](docker-compose.yml).


### 2.2. Requirements

- **Docker Desktop**: compilation runs inside a Debian + Rust Nightly docker container
- **Python 3.11+**: build and docs scripts
- **QEMU** for the arch you are targeting
- **OVMF firmware** (`code.fd` + `vars.fd`) placed in [run/deps/ovmf/](run/deps/ovmf) -- available from [rust-osdev/ovmf-prebuilt](https://github.com/rust-osdev/ovmf-prebuilt)

### 2.3. Configuration

[run/config/build.toml](run/config/build.toml) controls build behavior. The file is gitignored, create it before first use.

```toml
[options]
profile = "debug"          # "debug" or "release"

[features]
debug-logging = true       # enable kdebug! log output
vmm-debug-logging = true   # enable kdebug! log output from the VMM

[extra_paths]
paths = [                  # directories appended to PATH at script startup
    "C:/Program Files/qemu",
    "C:/Program Files/Docker/Docker/resources/bin",
]
```

You can omit `[extra_paths]` entirely if everything is already on your PATH.

### 2.4. Build Scripts

### `scripts/<arch>/build.py` -- Building and Running

```
python scripts/<arch>/build.py build   # compile kernel + create ISO
python scripts/<arch>/build.py run     # launch QEMU with UEFI firmware
python scripts/<arch>/build.py all     # build then run
python scripts/<arch>/build.py clean   # delete build/ and target/
```

Compilation happens inside Docker; QEMU runs natively on the host.

### [`scripts/docs.py`](scripts/docs.py) -- Building Documentation

```
python scripts/docs.py build   # generate rustdoc for src/kernel inside Docker
python scripts/docs.py open    # open the generated docs in the browser
python scripts/docs.py all     # build then open (default)
python scripts/docs.py clean   # delete generated docs
```

---

This project is licensed under the **GNU General Public License v3.0** (GPL-3.0-only). See [LICENSE](LICENSE) for details.
<br>
To contribute to Ferrite, refer to [CONTRIBUTING.md](CONTRIBUTING.md).
