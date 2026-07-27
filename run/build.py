"""
File:
    build.py

Authors:
    MarioS271

Copyright:
    SPDX-License-Identifier: GPL-3.0-only

Description:
    Build script for ferrite_os — Rust x86-64 bare metal OS.
    Compilation + ISO creation runs inside Docker.
    QEMU runs natively on the host (Windows, Linux, macOS).

Usage:
    python build.py [build | run | all | clean]
"""

import subprocess
import shutil
import sys
import os
import stat
import json
import hashlib
import tomllib
import socket
import time
from pathlib import Path

# ─── config ───────────────────────────────────────────────────────────────────

ROOT  = Path(__file__).parent.parent
BUILD = ROOT / "build"

ISO   = BUILD / "ferrite_os.iso"
CACHE = BUILD / ".build_cache.json"

# Path to the kernel ELF *inside the container*
# (target/ lives in a docker volume — it is not visible on the host)
KERNEL_ELF_CONTAINER = "/ferrite_os/target/x86_64-unknown-none/debug/kernel"

# Path to the OVMF dependencies
OVMF_DIR    = ROOT / "run" / "dependencies" / "ovmf"
OVMF_CODE   = OVMF_DIR / "code.fd"
OVMF_VARS   = OVMF_DIR / "vars.fd"

# Directories never scanned for source changes
IGNORED_DIRS = {"build", "target", ".git", ".venv", "node_modules"}

# Other global vars
CONTAINER_NAME  = "ferrite_os"
TCP_SERIAL_PORT = 4231
HOST            = "localhost"
RETRY_DELAY     = 1.0

def load_config() -> dict:
    """
    Load build.toml if present.

    Only [extra_paths] is read, which exists so Windows can find qemu/docker
    without a global PATH entry. Elsewhere it is optional, so a missing file
    is not an error.
    """
    cfg_path = ROOT / "run" / "configs" / "build.toml"
    if not cfg_path.exists():
        return {}
    try:
        with open(cfg_path, "rb") as f:
            return tomllib.load(f)
    except tomllib.TOMLDecodeError as e:
        print(f"  ! Ignoring malformed {cfg_path}: {e}")
        return {}

_CFG        = load_config()
EXTRA_PATHS = _CFG.get("extra_paths", {}).get("paths", [])

def tracked_sources() -> list:
    """Source files whose contents decide whether a rebuild is needed."""
    patterns = ("*.rs", "Cargo.toml", "*.ld", "*.conf", "*.cfg", "config.toml")
    found = set()
    for pattern in patterns:
        for f in ROOT.rglob(pattern):
            if IGNORED_DIRS.isdisjoint(f.relative_to(ROOT).parts):
                found.add(f)
    return sorted(found)

# ─── helpers ──────────────────────────────────────────────────────────────────

def banner(msg: str):
    print(f"\n{'─'*50}\n  {msg}\n{'─'*50}")

def patch_path():
    current = os.environ.get("PATH", "")
    for p in EXTRA_PATHS:
        if p not in current and os.path.exists(p):
            os.environ["PATH"] = p + os.pathsep + current

def rmtree(path: Path):
    """shutil.rmtree that survives read-only files (cargo does this on Windows)."""
    def handler(func, target, _exc):
        os.chmod(target, stat.S_IWRITE)
        func(target)

    if sys.version_info >= (3, 12):
        shutil.rmtree(path, onexc=handler)
    else:
        shutil.rmtree(path, onerror=handler)

def qemu_file(path: Path) -> str:
    """QEMU splits -drive options on commas, so commas in paths must be doubled."""
    return str(path).replace(",", ",,")

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
        print("\n  On Windows you can list the install dirs in")
        print("  run/configs/build.toml instead of the global PATH:")
        print("")
        print("    [extra_paths]")
        print("    paths = [")
        print("        \"C:/Program Files/qemu\",")
        print("        \"C:/Program Files/Docker/Docker/resources/bin\",")
        print("    ]")
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
    h = hashlib.md5(usedforsecurity=False)
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
    BUILD.mkdir(parents=True, exist_ok=True)
    CACHE.write_text(json.dumps(data, indent=2))

def build_hash() -> dict:
    # Keys are repo-relative posix paths so the cache is not machine-specific
    return {f.relative_to(ROOT).as_posix(): hash_file(f) for f in tracked_sources()}

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
            print(f"    ~ {f}")
        return True
    print("  ✓ Nothing changed — skipping build")
    return False

# ─── container lifecycle ──────────────────────────────────────────────────────

def container_running() -> bool:
    result = subprocess.run(
        ["docker", "inspect", "-f", "{{.State.Running}}", CONTAINER_NAME],
        capture_output=True, text=True
    )
    return result.returncode == 0 and result.stdout.strip() == "true"

def ensure_container_running():
    """Start the container if it isn't already running."""
    if container_running():
        print(f"  ✓ Container '{CONTAINER_NAME}' is running")
        return
    print(f"  Container '{CONTAINER_NAME}' not running — starting...")
    # cwd=ROOT so the compose file is found regardless of where this was invoked
    run(["docker", "compose", "up", "-d", "--build"], cwd=ROOT)

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

    # 4. Copy Limine config (strip CRLF so Limine can parse it)
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

    cmd = [
        "qemu-system-x86_64",
        "-cdrom",     str(ISO),
        "-m",         "1G",
        "-vga",       "std",
        "-serial",    f"tcp::{TCP_SERIAL_PORT},server,nowait",
        "-drive",     f"if=pflash,format=raw,readonly=on,file={qemu_file(OVMF_CODE)}",
        "-drive",     f"if=pflash,format=raw,file={qemu_file(OVMF_VARS)}",
    ]
    print(f"  >> {' '.join(str(c) for c in cmd)}")
    qemu = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        stream_serial(qemu)
    except KeyboardInterrupt:
        pass
    finally:
        if qemu.poll() is None:
            qemu.terminate()
            try:
                qemu.wait(timeout=5)
            except subprocess.TimeoutExpired:
                qemu.kill()

def stream_serial(qemu_proc):
    print(f"  Waiting for serial on {HOST}:{TCP_SERIAL_PORT}...")
    while True:
        try:
            sock = socket.create_connection((HOST, TCP_SERIAL_PORT), timeout=1)
            sock.settimeout(None)
            break
        except (ConnectionRefusedError, TimeoutError, OSError):
            if qemu_proc.poll() is not None:
                print("  ✗ QEMU exited before serial port opened")
                return
            time.sleep(RETRY_DELAY)

    banner(f"Serial Output  [{HOST}:{TCP_SERIAL_PORT}]")
    try:
        with sock:
            while True:
                data = sock.recv(4096)
                if not data:
                    break
                sys.stdout.buffer.write(data)
                sys.stdout.buffer.flush()
    except (KeyboardInterrupt, ConnectionResetError):
        pass
    print("\n  Serial connection closed")

def clean():
    banner("Cleaning Build")
    removed = False

    # target/ is a docker volume, so it has to be cleaned inside the container
    if container_running():
        run_in_container("cd /ferrite_os && cargo clean")
        removed = True
    else:
        print(f"  Container '{CONTAINER_NAME}' not running — skipping cargo clean")

    for d in [BUILD, ROOT / "target"]:
        if d.exists():
            rmtree(d)
            print(f"  ✓ Deleted {d}")
            removed = True

    if not removed:
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
        print(f"\n  Usage: python build.py [{' | '.join(COMMANDS)}]")
        print( "  Commands:")
        print( "    build  — compile kernel + create ISO in Docker (skips if unchanged)")
        print( "    run    — launch QEMU with the ISO (host-native)")
        print( "    all    — build + run")
        print( "    clean  — cargo clean in the container + delete build/")
        sys.exit(1)

    COMMANDS[sys.argv[1]]()

if __name__ == "__main__":
    main()
