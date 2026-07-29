# FerriteOS

A bare-metal x86-64 kernel written from scratch in Rust, targeting UEFI systems with the [Limine v8.x](https://github.com/limine-bootloader/limine) bootloader.
Long-term goal: full Linux x86_64 ABI compatibility — run unmodified Linux ELF binaries on a kernel written entirely in Rust.

`no_std`, `x86_64-unknown-none`. No OS underneath. Single Cargo workspace member at `src/kernel`.
Architecture-specific code is isolated under `arch/<arch>/`, `logging/<arch>/`, `mem/<arch>/`; the parent `mod.rs` of each selects the right submodule at compile time via `#[cfg(target_arch)]` and re-exports it, so the rest of the kernel uses architecture-independent paths.

Subsystem documentation lives in [`docs/`](docs/), mirroring `src/kernel/src/`.

---

## Requirements

- **Docker Desktop** — compilation runs inside a Debian + nightly Rust container
- **Python 3.11+** — build and docs scripts
- **QEMU** with x86-64 support
- **OVMF firmware** (`code.fd` + `vars.fd`) placed in `run/deps/ovmf/` — available from [rust-osdev/ovmf-prebuilt](https://github.com/rust-osdev/ovmf-prebuilt)

---

## Configuration

`run/config/build.toml` controls build behavior. The file is not committed — create it before first use.

```toml
[options]
profile = "debug"       # "debug" or "release"

[features]
debug-logging = true    # enable kdebug! log output

[extra_paths]
paths = [               # directories appended to PATH at script startup
    "C:/Program Files/qemu",
    "C:/Program Files/Docker/Docker/resources/bin",
]
```

All sections are optional. Omit `[extra_paths]` entirely if everything is already on your PATH.

---

## Scripts

### `scripts/x86_64/build.py` — build and run

```
python scripts/x86_64/build.py build   # compile kernel + create ISO (skips unchanged sources)
python scripts/x86_64/build.py run     # launch QEMU with UEFI firmware
python scripts/x86_64/build.py all     # build then run
python scripts/x86_64/build.py clean   # delete build/ and target/
```

Compilation happens inside Docker; QEMU runs natively on the host. Incremental builds use MD5 hashing to skip the Docker step when nothing has changed. There is no `cargo test` — the bare-metal target does not support it.

### `scripts/docs.py` — rustdoc

```
python scripts/docs.py build   # generate rustdoc for src/kernel inside Docker
python scripts/docs.py open    # open the generated docs in the browser
python scripts/docs.py all     # build then open (default)
python scripts/docs.py clean   # delete generated docs
```

---

## License

GPL-3.0-only. See [LICENSE](LICENSE).
