# Ferrite OS — Architecture

> A Linux-compatible kernel written in Rust with a native namespace-based VFS, targeting full Linux ABI compatibility while providing a cleaner native API.

---

## Goals

- Full Linux x86_64 ABI compatibility (run unmodified Linux ELF binaries)
- Native namespace-based VFS (`vdev:/`, `dev:/`, `proc:/`, `sys:/`, `file:/`)
- Both classic Unix paths and native namespace paths accepted everywhere
- Rust throughout the kernel
- Eventually: Wayland compositor, GNOME, Wine integration

---

## Multi-Arch Code Organization

`arch/`, `logging/`, and `mem/` each contain an `x86_64/` and `aarch64/` subdirectory. The parent `mod.rs` in each selects the correct submodule at compile time via `#[cfg(target_arch)]` and re-exports its public items with `pub(crate) use arch_name::*`, so all call sites use the parent path (e.g. `arch::instructions::halt_cpu()`). aarch64 modules are stubs that emit `compile_error!` to catch accidental cross-compilation early. New architectures follow the same pattern: add a subdirectory, implement the required interface, add the `#[cfg]` pair in the parent.

---

## Boot Sequence

```
Limine bootloader
    ↓
kmain()
    ├── save Limine responses        (memory map, HHDM offset, kernel address)
    ├── set HHDM offset              (phys ↔ virt translation base)
    ├── init GDT + TSS               (correct segment order for syscall/sysret)
    ├── init IDT                     (exception + interrupt handlers)
    ├── cpu_init()                   (SSE, FSGSBASE, EFER/SCE bit)
    ├── init_syscalls()              (STAR, LSTAR, SFMASK MSRs)
    ├── parse memory map             (classify usable/reserved/reclaimable)
    ├── init PMM                     (bitmap frame allocator)
    ├── init VMM                     (page tables, HHDM mapping)
    ├── init heap                    (kernel slab/linked-list allocator)
    ├── init scheduler               (round-robin to start)
    ├── init VFS + namespaces        (register all namespace handlers)
    ├── init TTY layer               (line discipline, termios)
    └── spawn init process (pid 1)
```

---

## Memory Layout

```
0x0000000000000000  ─┐
                     │  Userspace (low half)
                     │  0x0000 - 0x7FFF FFFF FFFF
0x00007FFFFFFFFFFF  ─┘

── non-canonical gap ──

0xFFFF800000000000  ─┐
                     │  HHDM (direct physical map)
                     │  entire physical RAM mapped here
0xFFFF887FFFFFFFFF  ─┘

0xFFFFFFFF80000000  ─┐
                     │  Kernel image
0xFFFFFFFFFFFFFFFF  ─┘
```

Physical ↔ virtual translation:
```
virt = phys + HHDM_OFFSET
phys = virt - HHDM_OFFSET
```

---

## PMM — Physical Memory Manager

**Design:** bitmap allocator. One bit per 4KB physical frame.

```
bit = 0  →  frame free
bit = 1  →  frame used
```

**Invariants:**
- Frame 0 is never allocated (null pointer safety)
- All frames default to used; only explicitly usable regions are freed
- Bitmap frames marked used immediately after placement
- Bootloader-reclaimable memory freed only after Limine data fully consumed

**Interface (entire public API):**
```
pub fn alloc() -> Option<PhysAddr>  // returns 4KB-aligned physical address
pub fn free(addr: PhysAddr)
pub fn stats() -> PmmStats          // for proc:/meminfo
```

---

## VMM — Virtual Memory Manager

Sits above PMM. Manages per-process virtual address spaces and the kernel's own mappings.

**Responsibilities:**
- Map/unmap/remap virtual → physical via page tables
- Handle page faults (CoW, stack growth, lazy allocation)
- Implement `mmap`, `munmap`, `mprotect`, `brk`
- Copy-on-write fork support

**Linux compat critical:**
```rust
// mmap flags match Linux uapi exactly
const MAP_SHARED:    u32 = 0x01;
const MAP_PRIVATE:   u32 = 0x02;
const MAP_FIXED:     u32 = 0x10;
const MAP_ANONYMOUS: u32 = 0x20;

// brk(0) returns current break
// brk(addr) returns current (unchanged) break on failure, not -1
```

---

## Process Model

```rust
pub struct Process {
    pid:         Pid,
    ppid:        Pid,
    pgid:        Pid,         // process group (job control)
    sid:         Pid,         // session (TTY ownership)
    uid:         u32,
    gid:         u32,
    euid:        u32,         // effective (for setuid)
    egid:        u32,
    umask:       u32,
    cwd:         InternalPath,
    root:        InternalPath, // chroot support
    files:       FileTable,    // fd table
    mm:          MemoryMap,    // virtual address space
    signals:     SignalState,
    personality: Personality,  // NATIVE or LinuxCompat
    children:    Vec<Pid>,
    threads:     Vec<Tid>,     // clone()/pthreads
}
```

**fork/exec/wait semantics match Linux exactly.**
- `fork`: CoW clone, child gets 0, parent gets child pid
- `execve`: replaces image, preserves pid/ppid/sid/pgid/open fds (except `O_CLOEXEC`)
- `waitpid`: supports `pid == -1`, `pid == 0`, `pid < -1`, `WNOHANG`, `WUNTRACED`

---

## Syscall Layer

Linux x86_64 ABI via `syscall`/`sysret`:

```
rax = syscall number
rdi rsi rdx r10 r8 r9 = arguments (note r10 not rcx)
return value in rax
```

Syscall numbers match Linux exactly. Priority syscalls for reaching a shell:

| Number | Name | Notes |
|--------|------|-------|
| 0 | read | |
| 1 | write | |
| 2 | open | path goes through namespace normalizer |
| 9 | mmap | |
| 12 | brk | returns old brk on failure |
| 57 | fork | CoW |
| 59 | execve | ELF loader |
| 60 | exit | |
| 158 | arch_prctl | `ARCH_SET_FS` for TLS — critical for musl/glibc |
| 231 | exit_group | |

`arch_prctl(ARCH_SET_FS)` is the first syscall musl makes. It must work or nothing runs.

---

## VFS + Namespace Layer

### Path Normalization

Single function called at every `open`/`stat`/`mkdir`/`rename` etc. Both syntaxes produce the same internal path — no translation layer needed, no per-process flags needed for path handling.

```
fn normalize(path: &str) -> InternalPath {
    match path {
        // virtual devices — classic unix paths
        p if p.starts_with("/dev/null")    => ip("vdev", "null"),
        p if p.starts_with("/dev/zero")    => ip("vdev", "zero"),
        p if p.starts_with("/dev/random")  => ip("vdev", "random"),
        p if p.starts_with("/dev/urandom") => ip("vdev", "urandom"),
        p if p.starts_with("/dev/tty")     => ip("vdev", "tty"),
        p if p.starts_with("/dev/pts")     => ip("vdev", &p[5..]),
        // real hardware — classic
        p if p.starts_with("/dev/")        => ip("dev",  &p[5..]),
        p if p.starts_with("/proc/")       => ip("proc", &p[6..]),
        p if p.starts_with("/sys/")        => ip("sys",  &p[4..]),
        p if p.starts_with("/")            => ip("file", &p[1..]),
        // native namespace syntax
        p if p.starts_with("vdev:/")       => ip("vdev", &p[6..]),
        p if p.starts_with("dev:/")        => ip("dev",  &p[5..]),
        p if p.starts_with("proc:/")       => ip("proc", &p[7..]),
        p if p.starts_with("sys:/")        => ip("sys",  &p[5..]),
        p if p.starts_with("file:/")       => ip("file", &p[6..]),
        p                                  => ip("file", p),
    }
}
```

`realpath()` always returns classic `/dev/` style so POSIX software that stores and reuses paths never breaks.

### Namespace Map

| Namespace | Classic path                                    | Purpose                          |
|-----------|-------------------------------------------------|----------------------------------|
| `vdev:/`  | `/dev/null` `/dev/zero` `/dev/tty` `/dev/pts/*` | Virtual/synthetic devices        |
| `dev:/`   | `/dev/sda` `/dev/dri/card0` `/dev/input/*`      | Real hardware devices            |
| `proc:/`  | `/proc/*`                                       | Process and kernel info          |
| `sys:/`   | `/sys/*`                                        | Hardware topology                |
| `net:/`   | `/sys/class/net/*` `/proc/net/*`                | Network interfaces (native only) |
| `file:/`  | `/`                                             | Regular filesystem               |

### Namespace Trait

```rust
pub trait Namespace: Send + Sync {
    fn open(&self, path: &str, flags: OpenFlags) -> Result<Box<dyn Fd>, VfsError>;
    fn stat(&self, path: &str) -> Result<Stat, VfsError>;
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError>;
    fn create(&self, path: &str, flags: OpenFlags) -> Result<(), VfsError> {
        Err(VfsError::ReadOnly)  // most namespaces are read-only by default
    }
}
```

### Reserved Names in `file:/`

These names cannot be created by userspace at the root of `file:/`:

```
dev  proc  sys  net  tmp  run  mnt
```

Enforced at `mkdir`, `rename`, `symlink`, `hardlink`. Returns `EACCES`. Also requires `Capabilities::CREATE_RESERVED_DIRS` to bypass (kernel/init only).

---

## TTY Layer

```rust
pub struct Tty {
    termios:          Termios,     // stty settings
    winsize:          Winsize,     // TIOCGWINSZ
    foreground_pgid:  Pid,         // job control
    session:          Sid,
    input_buf:        LineBuffer,
    canon_buf:        LineBuffer,  // canonical/cooked mode
    output_buf:       RingBuffer,
}
```

`termios.cc` special characters implement Ctrl+C (`VINTR`), Ctrl+D (`VEOF`), backspace (`VERASE`), Ctrl+Z (`VSUSP`) etc.

PTY pair (`vdev:/ptmx` → `vdev:/pts/N`) for terminal emulators.

### Signal mapping from TTY

```
VINTR (Ctrl+C)  → SIGINT  → foreground process group
VQUIT (Ctrl+\)  → SIGQUIT → foreground process group
VSUSP (Ctrl+Z)  → SIGTSTP → foreground process group
```

---

## Signal Layer

```rust
pub struct SignalState {
    pending:  SignalSet,
    blocked:  SignalSet,           // sigmask
    handlers: [SigAction; 64],
    altstack: Option<SigAltStack>, // sigaltstack()
}

pub struct SigAction {
    handler: SigHandler,           // SIG_DFL | SIG_IGN | fn ptr
    mask:    SignalSet,            // additional mask during handler
    flags:   SigFlags,            // SA_RESTART, SA_SIGINFO, SA_NODEFER
}
```

Delivery happens at syscall exit and interrupt return, never mid-syscall. `SA_RESTART` transparently restarts interrupted blocking syscalls.

### Exception → Signal mapping

| Vector | Exception | Signal |
|--------|-----------|--------|
| #0 | Divide by zero | SIGFPE |
| #6 | Invalid opcode | SIGILL |
| #11 | Segment not present | SIGBUS |
| #13 | General protection fault | SIGSEGV |
| #14 | Page fault | SIGSEGV or grow stack |
| #19 | SIMD float exception | SIGFPE |

---

## Linux Binary Detection

Set at `execve` time, inherited across `fork`, cleared on new `execve`.

```rust
pub enum Personality {
    Native = 0,
    LinuxCompat = 1 << 0,
}
```

Detection priority:
1. `PT_INTERP` segment = `/lib64/ld-linux-x86-64.so.2` or `/lib/ld-musl-x86_64.so.1` → strong signal
2. `.note.ABI-tag` section with OS = 0 (Linux) → confirms
3. ELF `os_abi` = `0x03` → confirms
4. None of the above → `NATIVE`

`LinuxCompat` enables nothing extra for path handling (normalizer handles both already). Reserved for future behavioral differences.

---

## proc:/ Output Formats

Must be byte-compatible with Linux. Programs parse these as text with specific field names and whitespace.

```
proc:/meminfo
    MemTotal:      16384000 kB
    MemFree:        8192000 kB
    MemAvailable:   9000000 kB

proc:/cpuinfo
    processor	: 0
    vendor_id	: GenuineIntel
    ...

proc:/self/maps
    7f4a3b200000-7f4a3b400000 r-xp 00000000 08:01 1234  /lib/libc.so

proc:/self/status
    Name:	bash
    Pid:	1234
    PPid:	1
    Uid:	1000 1000 1000 1000
```

---

## fsh — Ferrite Shell

Native shell. Uses classic paths in its own code (works either way).

Required for basic operation:
- `fork` + `exec` for running programs
- `pipe()` for `|`
- `dup2()` for `<` `>` `>>`
- `waitpid()` for foreground jobs
- `SIGCHLD` for background jobs
- `tcsetpgrp()` for job control
- termios raw mode for line editing
- `$PATH` resolution
- Builtins: `cd` `pwd` `echo` `export` `exit` `jobs` `fg` `bg` `source`

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
                                        → fsh
                                            → dev:/ (DRM/KMS, input)
                                            → real filesystem (ext2/fat)
                                            → networking
                                            → Wayland/Weston
```

---

## Compatibility Targets (in order)

| Milestone                     | What it proves                          |
|-------------------------------|-----------------------------------------|
| musl statically linked binary | PMM + VMM + syscalls + ELF loader work  |
| busybox sh                    | procfs, signals, TTY line discipline    |
| bash + coreutils              | Full POSIX process model, job control   |
| Xorg/twm                      | DRM/KMS driver, input subsystem         |
| GTK3 app                      | D-Bus, fontconfig, GIO                  |
| XFCE                          | Light desktop, reduced systemd coupling |
| GNOME                         | Full stack                              |