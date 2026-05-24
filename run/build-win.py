"""
File:
    build-win.py

Authors:
    MarioS271

Copyright:
    SPDX-License-Identifier: GPL-3.0-only

Description:
    Build script for ferrite_os — Rust x86-64 bare metal OS.
    Compilation + ISO creation runs inside Docker.
    QEMU runs natively.

Usage:
    python build-win.py [config | build | run | all | clean]
"""

import subprocess
import shutil
import sys
import os
import json
import hashlib
import tomllib
from pathlib import Path

# ─── config ───────────────────────────────────────────────────────────────────

ROOT  = Path(__file__).parent.parent
BUILD = ROOT / "build"

ISO   = BUILD / "ferrite_os.iso"
CACHE = BUILD / ".build_cache.json"

# Path to the kernel ELF *inside the container*
KERNEL_ELF_CONTAINER = "/ferrite_os/target/x86_64-unknown-none/debug/kernel"

# Path to the kernel ELF on the host (via volume mount)
KERNEL_ELF_HOST = ROOT / "target" / "x86_64-unknown-none" / "debug" / "kernel"

# Path to the OVMF dependencies
OVMF_DIR    = ROOT / "run" / "dependencies" / "ovmf"
OVMF_CODE   = OVMF_DIR / "code.fd"
OVMF_VARS   = OVMF_DIR / "vars.fd"

# Other global vars
CONTAINER_NAME = "ferrite_os"
TCP_SERIAL_PORT = 4231

def load_config() -> dict:
    cfg_path = ROOT / "run" / "configs" / "build.toml"
    if not cfg_path.exists():
        print(f"  ✗ build.toml not found at {cfg_path}")
        print( "  Create one — example:")
        print( "")
        print( "    [extra_paths]")
        print( "    paths = [")
        print( "        \"C:/Program Files/qemu\",")
        print( "        \"C:/Program Files/Docker/Docker/resources/bin\",")
        print( "    ]")
        sys.exit(1)
    with open(cfg_path, "rb") as f:
        return tomllib.load(f)

_CFG        = load_config()
EXTRA_PATHS = _CFG.get("extra_paths", {}).get("paths", [])

TRACKED_SOURCES = (
        list(ROOT.rglob("*.rs"))      +
        list(ROOT.rglob("Cargo.toml"))+
        list(ROOT.rglob("*.ld"))      +
        list(ROOT.rglob("*.conf"))    +
        list(ROOT.rglob("*.cfg"))     +
        list(ROOT.rglob("config.toml"))
)

# ─── helpers ──────────────────────────────────────────────────────────────────

def banner(msg: str):
    print(f"\n{'─'*50}\n  {msg}\n{'─'*50}")

def patch_path():
    current = os.environ.get("PATH", "")
    for p in EXTRA_PATHS:
        if p not in current and os.path.exists(p):
            os.environ["PATH"] = p + ";" + current

def run(cmd: list, **kwargs):
    """Run a command, print it, die on failure."""
    print(f"  >> {' '.join(str(c) for c in cmd)}")
    result = subprocess.run(cmd, **kwargs)
    if result.returncode != 0:
        print(f"  FAILED (exit {result.returncode})")
        sys.exit(result.returncode)

def run_in_container(shell_cmd: str):
    """Run a shell command inside the running ferrite_os container."""
    run(["docker", "exec", CONTAINER_NAME, "bash", "-c", shell_cmd])

def check_dependencies():
    banner("Checking Dependencies")
    deps = ["docker", "qemu-system-x86_64"]
    missing = [d for d in deps if shutil.which(d) is None]
    if missing:
        print("  Missing tools (not in PATH):")
        for m in missing:
            print(f"  ✗ {m}")
        sys.exit(1)
    for d in deps:
        print(f"  ✓ {d}")

def check_ovmf():
    banner("Checking OVMF")
    ok = True
    for f in [OVMF_CODE, OVMF_VARS]:
        if f.exists():
            print(f"  ✓ {f}")
        else:
            print(f"  ✗ {f}  ← not found")
            ok = False
    if not ok:
        print("\n  Download OVMF from:")
        print("  https://github.com/rust-osdev/ovmf-prebuilt/releases")
        print("  Place code.fd and vars.fd in run/dependencies/ovmf/")
        sys.exit(1)

def list_config_vars():
    banner("Config")
    vars = {
        "ROOT":      ROOT,
        "BUILD":     BUILD,
        "ISO":       ISO,
        "OVMF_CODE": OVMF_CODE,
        "OVMF_VARS": OVMF_VARS,
    }
    for name, value in vars.items():
        exists = "✓" if Path(str(value)).exists() else "✗"
        print(f"  {exists} {name} = {value}")

# ─── cache ────────────────────────────────────────────────────────────────────

def hash_file(path: Path) -> str:
    h = hashlib.md5()
    try:
        h.update(path.read_bytes())
    except FileNotFoundError:
        return ""
    return h.hexdigest()

def load_cache() -> dict:
    try:
        return json.loads(CACHE.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return {}

def save_cache(data: dict):
    BUILD.mkdir(exist_ok=True)
    CACHE.write_text(json.dumps(data, indent=2))

def build_hash() -> dict:
    return {str(f): hash_file(f) for f in TRACKED_SOURCES if f.exists()}

def needs_rebuild() -> bool:
    if not ISO.exists():
        print("  No ISO found — full build required")
        return True
    current = build_hash()
    cached  = load_cache()
    changed = [f for f, h in current.items() if cached.get(f) != h]
    if changed:
        print("  Changed files:")
        for f in changed:
            try:
                print(f"    ~ {Path(f).relative_to(ROOT)}")
            except ValueError:
                print(f"    ~ {f}")
        return True
    print("  ✓ Nothing changed — skipping build")
    return False

# ─── container lifecycle ──────────────────────────────────────────────────────

def ensure_container_running():
    """Start the container if it isn't already running."""
    result = subprocess.run(
        ["docker", "inspect", "-f", "{{.State.Running}}", CONTAINER_NAME],
        capture_output=True, text=True
    )
    if result.returncode != 0 or result.stdout.strip() != "true":
        print(f"  Container '{CONTAINER_NAME}' not running — starting...")
        run(["docker", "compose", "up", "-d", "--build"])
    else:
        print(f"  ✓ Container '{CONTAINER_NAME}' is running")

# ─── build steps ──────────────────────────────────────────────────────────────

def do_build():
    banner("Building Kernel (in container)")

    if not needs_rebuild():
        return

    ensure_container_running()

    # 1. Compile the kernel ELF
    run_in_container(
        "cd /ferrite_os && "
        "cargo build --target x86_64-unknown-none "
        "--manifest-path src/kernel/Cargo.toml"
    )

    # 2. Create ISO directory structure inside the container
    run_in_container(
        "mkdir -p /ferrite_os/build/iso/boot/limine && "
        "mkdir -p /ferrite_os/build/iso/EFI/BOOT"
    )

    # 3. Copy kernel ELF into ISO root
    run_in_container(
        f"cp {KERNEL_ELF_CONTAINER} /ferrite_os/build/iso/kernel"
    )

    # 4. Copy Limine config (strip Windows CRLF so Limine can parse it)
    run_in_container(
        "sed 's/\\r//' /ferrite_os/run/configs/limine.conf "
        "> /ferrite_os/build/iso/boot/limine/limine.conf"
    )

    # 5. Copy Limine UEFI binaries
    run_in_container(
        "cp /opt/limine/BOOTX64.EFI   /ferrite_os/build/iso/EFI/BOOT/ && "
        "cp /opt/limine/BOOTIA32.EFI  /ferrite_os/build/iso/EFI/BOOT/ && "
        "cp /opt/limine/limine-uefi-cd.bin /ferrite_os/build/iso/boot/limine/"
    )

    # 6. Build the ISO with xorriso (UEFI only — no BIOS El Torito)
    run_in_container(
        "xorriso -as mkisofs "
        "-no-emul-boot "
        "--efi-boot boot/limine/limine-uefi-cd.bin "
        "-efi-boot-part --efi-boot-image "
        "--protective-msdos-label "
        "/ferrite_os/build/iso "
        "-o /ferrite_os/build/ferrite_os.iso"
    )

    if not ISO.exists():
        print("  ✗ ISO not found after build — something went wrong")
        sys.exit(1)

    print(f"  ✓ ISO: {ISO}")
    save_cache(build_hash())
    print("  ✓ Build cache updated")

def run_qemu():
    banner("Launching QEMU")
    if not ISO.exists():
        print("  ✗ No ISO found. Run build first.")
        sys.exit(1)

    run([
        "qemu-system-x86_64",
        "-cdrom",     str(ISO),
        "-m",         "1G",
        "-vga",       "std",
        "-serial",    f"tcp::{TCP_SERIAL_PORT},server,nowait",
        "-drive",     f"if=pflash,format=raw,readonly=on,file={OVMF_CODE}",
        "-drive",     f"if=pflash,format=raw,file={OVMF_VARS}",
    ])

def clean():
    banner("Cleaning Build")
    removed = []
    for d in [BUILD, ROOT / "target"]:
        if d.exists():
            shutil.rmtree(d)
            removed.append(d)
    if removed:
        for d in removed:
            print(f"  ✓ Deleted {d}")
    else:
        print("  Nothing to clean")

# ─── commands ─────────────────────────────────────────────────────────────────

def cmd_build():
    check_dependencies()
    do_build()

def cmd_run():
    check_dependencies()
    check_ovmf()
    run_qemu()

def cmd_all():
    check_dependencies()
    check_ovmf()
    do_build()
    run_qemu()

def cmd_clean():
    clean()

COMMANDS = {
    "build": cmd_build,
    "run":   cmd_run,
    "all":   cmd_all,
    "clean": cmd_clean,
}

def main():
    sys.stdout.reconfigure(encoding="utf-8")
    patch_path()
    list_config_vars()

    if len(sys.argv) < 2 or sys.argv[1] not in COMMANDS:
        print(f"\n  Usage: python build-win.py [{' | '.join(COMMANDS)}]")
        print( "  Commands:")
        print( "    build  — compile kernel + create ISO in Docker (skips if unchanged)")
        print( "    run    — launch QEMU with the ISO (Windows-native)")
        print( "    all    — build + run")
        print( "    clean  — delete build/ and target/")
        sys.exit(1)

    COMMANDS[sys.argv[1]]()

if __name__ == "__main__":
    main()