"""
File:
    build.py

Authors:
    MarioS271

Copyright:
    SPDX-License-Identifier: GPL-3.0-only

Description:
    Build script for ferrite_os — Rust x86_64 bare metal OS.
    Compilation + ISO creation runs inside Docker.
    QEMU runs natively on the host (Windows, Linux, macOS).

Usage:
    python build.py [build | run | all | clean]
"""

import re
import socket
import subprocess
import sys
import time
from pathlib import Path

# scripts/ holds the shared helpers; it is not on sys.path when this script
# is invoked directly, since Python only adds scripts/x86_64/.
sys.path.insert(0, str(Path(__file__).parent.parent))

import lib
from lib import BUILD, ROOT, banner, run_in_container

# ─── config ───────────────────────────────────────────────────────────────────

ISO = BUILD / "ferrite_os.iso"
LOG = ROOT / "run" / "latest.log"

# Path to the OVMF deps
OVMF_DIR  = ROOT / "run" / "deps" / "ovmf"
OVMF_CODE = OVMF_DIR / "code.fd"
OVMF_VARS = OVMF_DIR / "vars.fd"

TCP_SERIAL_PORT = 4231
HOST            = "localhost"
RETRY_DELAY     = 1.0

def options_error(reason: str):
    """Print why the [options] config is unusable, suggest a default, and exit."""
    print(f"  ✗ {reason}")
    print("\n  Add a valid [options] section to run/config/build.toml:")
    print("")
    print("    [options]")
    print("    profile = \"debug\"    # \"debug\" or \"release\"")
    print("")
    print("    [features]")
    print("    debug-kprint = true  # set false to omit the --features flag")
    sys.exit(1)

def load_options(cfg: dict) -> str:
    """Read [options].profile. Errors on absence or invalid value."""
    options = cfg.get("options")
    if options is None:
        options_error("No [options] section in build.toml")

    profile = options.get("profile")
    if profile is None:
        options_error("[options].profile is missing")
    if profile not in ("debug", "release"):
        options_error(f"[options].profile must be \"debug\" or \"release\", got {profile!r}")

    return profile

def load_features(cfg: dict) -> list:
    """Read [features] and return names where the value is true. Missing section = no features."""
    section = cfg.get("features", {})
    if not isinstance(section, dict):
        options_error("[features] must be a TOML table of feature-name = true/false pairs")
    bad = {k: v for k, v in section.items() if not isinstance(v, bool)}
    if bad:
        options_error(f"[features] values must be true or false, got: {bad}")
    return [name for name, enabled in section.items() if enabled]

PROFILE  = load_options(lib.CFG)
FEATURES = load_features(lib.CFG)

# Path to the kernel ELF *inside the container*
# (target/ lives in a docker volume — it is not visible on the host)
# The profile ("debug" / "release") selects the cargo output subdirectory.
KERNEL_ELF_CONTAINER = f"/ferrite_os/target/x86_64-unknown-none/{PROFILE}/kernel"

# ─── checks ───────────────────────────────────────────────────────────────────

def check_deps():
    lib.check_dependencies(["docker", "qemu-system-x86_64"])

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
        print("  Place code.fd and vars.fd in run/deps/ovmf/")
        sys.exit(1)

# ─── build steps ──────────────────────────────────────────────────────────────

def do_build():
    banner("Building Kernel (in container)")

    lib.ensure_container_running()

    cargo_cmd = (
        "cd /ferrite_os && "
        "cargo build --target x86_64-unknown-none "
        "--manifest-path src/kernel/Cargo.toml"
    )
    if PROFILE == "release":
        cargo_cmd += " --release"
    if FEATURES:
        cargo_cmd += " --features " + ",".join(FEATURES)
    run_in_container(cargo_cmd)

    # Assemble the ISO tree and hand it to xorriso. One docker exec, because
    # every step here is a cheap file copy and a round trip per copy costs
    # more than the copy does. UEFI only — no BIOS El Torito.
    run_in_container(
        "set -e && "
        "cd /ferrite_os/build && "
        "rm -rf iso && "
        "mkdir -p iso/boot/limine iso/EFI/BOOT && "
        f"cp {KERNEL_ELF_CONTAINER} iso/ferrite && "
        # strip CRLF so Limine can parse the config
        "sed 's/\\r//' /ferrite_os/run/config/limine.conf > iso/boot/limine/limine.conf && "
        "cp /opt/limine/BOOTX64.EFI /opt/limine/BOOTIA32.EFI iso/EFI/BOOT/ && "
        "cp /opt/limine/limine-uefi-cd.bin iso/boot/limine/ && "
        "xorriso -as mkisofs "
        "-no-emul-boot "
        "--efi-boot boot/limine/limine-uefi-cd.bin "
        "-efi-boot-part --efi-boot-image "
        "--protective-msdos-label "
        "iso -o ferrite_os.iso"
    )

    if not ISO.exists():
        print("  ✗ ISO not found after build — something went wrong")
        sys.exit(1)

    print(f"  ✓ ISO: {ISO}")

def qemu_file(path: Path) -> str:
    """QEMU splits -drive options on commas, so commas in paths must be doubled."""
    return str(path).replace(",", ",,")

def run_qemu():
    banner("Launching QEMU")
    if not ISO.exists():
        print("  ✗ No ISO found. Run build first.")
        sys.exit(1)

    cmd = [
        "qemu-system-x86_64",
        "-cdrom",     str(ISO),
        "-m",         "4G",
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

def clean():
    banner("Cleaning")
    lib.ensure_container_running()
    run_in_container("rm -rf /ferrite_os/target/* /ferrite_os/build/*")
    print("  ✓ Clean done")

# ─── serial streaming ─────────────────────────────────────────────────────────

# The kernel emits plain text over serial; every escape sequence on the wire
# comes from the UEFI firmware before the kernel starts. Dropping all of them
# keeps the host terminal intact, so no terminal state has to be saved and
# restored afterwards.
ESCAPE_SEQUENCE = re.compile(
    rb"\x1b(?:"
    rb"\[[0-?]*[ -/]*[@-~]"             # CSI — cursor moves, screen clears, SGR
    rb"|\][^\x07\x1b]*(?:\x07|\x1b\\)"  # OSC — window titles, terminated by BEL/ST
    rb"|[\x20-\x2f]+[\x30-\x7e]"       # nF  — charset designation (ESC ( B)
    # Fp/Fe/Fs single-character escapes, including RIS (ESC c) which would reset
    # the terminal outright. The CSI (0x5b) and OSC (0x5d) introducers are cut
    # out: they belong to the branches above, and swallowing a bare "ESC [" here
    # would leak the rest of a split sequence into the terminal as plain text.
    rb"|[\x30-\x5a\x5c\x5e-\x7e]"
    rb")"
)

# Everything below 0x20 except tab / newline / carriage return, plus DEL
CONTROL_CHARS = re.compile(rb"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]")

# Longest partial escape sequence held across a chunk boundary before giving up
MAX_PENDING_ESCAPE = 32

def sanitize(data: bytes, pending: bytes) -> tuple:
    """
    Strip terminal control sequences from a serial chunk.

    Returns the printable bytes plus any trailing partial escape sequence,
    which is prepended to the next chunk instead of being printed as garbage.
    """
    buf = ESCAPE_SEQUENCE.sub(b"", pending + data)

    # Any ESC still left here started a sequence that recv() cut in half, so
    # hold it back for the next chunk instead of printing it half-decoded.
    # Complete sequences are already gone, so this never withholds real output.
    start = buf.rfind(b"\x1b")
    pending = b""
    if start != -1 and len(buf) - start < MAX_PENDING_ESCAPE:
        buf, pending = buf[:start], buf[start:]

    return CONTROL_CHARS.sub(b"", buf), pending

def connect_serial(qemu_proc):
    """Wait for QEMU to open its serial port. Returns None if QEMU died first."""
    print(f"  Waiting for serial on {HOST}:{TCP_SERIAL_PORT}...")
    while True:
        try:
            sock = socket.create_connection((HOST, TCP_SERIAL_PORT), timeout=1)
            sock.settimeout(None)
            return sock
        except (ConnectionRefusedError, TimeoutError, OSError):
            if qemu_proc.poll() is not None:
                print("  ✗ QEMU exited before serial port opened")
                return None
            time.sleep(RETRY_DELAY)

def stream_serial(qemu_proc):
    sock = connect_serial(qemu_proc)
    if sock is None:
        return

    LOG.parent.mkdir(parents=True, exist_ok=True)
    print(f"  Logging serial output to {LOG}")

    banner(f"Serial Output  [{HOST}:{TCP_SERIAL_PORT}]")
    pending = b""
    try:
        # The log keeps the raw stream; only the terminal gets the filtered one
        with sock, LOG.open("wb") as log_file:
            while True:
                data = sock.recv(4096)
                if not data:
                    break

                log_file.write(data)
                log_file.flush()

                printable, pending = sanitize(data, pending)
                sys.stdout.buffer.write(printable)
                sys.stdout.buffer.flush()
    except (KeyboardInterrupt, ConnectionResetError):
        pass

    print("\n  Serial connection closed")
    print(f"  Log saved to {LOG}")

# ─── commands ─────────────────────────────────────────────────────────────────

COMMANDS = {
    "build": ("compile kernel + create ISO in Docker",        [check_deps, do_build]),
    "run":   ("launch QEMU with the ISO (host-native)",       [check_deps, check_ovmf, run_qemu]),
    "all":   ("build + run",                                  [check_deps, check_ovmf, do_build, run_qemu]),
    "clean": ("cargo clean in the container + delete build/", [clean]),
}

def main():
    sys.stdout.reconfigure(encoding="utf-8")
    lib.patch_path()
    lib.list_config_vars(
        {
            "ROOT":      ROOT,
            "BUILD":     BUILD,
            "ISO":       ISO,
            "OVMF_CODE": OVMF_CODE,
            "OVMF_VARS": OVMF_VARS,
        },
        {"profile": PROFILE, "features": FEATURES},
    )
    lib.dispatch("build.py", COMMANDS)

if __name__ == "__main__":
    main()
