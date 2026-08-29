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

ROOT  = Path(__file__).parent.parent.parent
BUILD = ROOT / "build"

ISO   = BUILD / "ferrite_os.iso"
CACHE = BUILD / ".build_cache.json"

LOG = ROOT / "run" / "latest.log"

# Path to the OVMF deps
OVMF_DIR    = ROOT / "run" / "deps" / "ovmf"
OVMF_CODE   = OVMF_DIR / "code.fd"
OVMF_VARS   = OVMF_DIR / "vars.fd"

# Directories never scanned for source changes
IGNORED_DIRS = {"build", "target", ".git", ".venv", "node_modules"}

# Other global vars
CONTAINER_NAME  = "ferrite_os"
TCP_SERIAL_PORT = 4231
HOST            = "localhost"
RETRY_DELAY     = 1.0

# Everything before this marker in the serial stream is UEFI/firmware noise
SERIAL_START_MARKER = b"I Hello, Ferrite"

def load_config() -> dict:
    """
    Load build.toml if present.

    Only [extra_paths] is read, which exists so Windows can find qemu/docker
    without a global PATH entry. Elsewhere it is optional, so a missing file
    is not an error.
    """
    cfg_path = ROOT / "run" / "config" / "build.toml"
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

def options_error(reason: str):
    """Print why the [options] config is unusable, suggest a default, and exit."""
    print(f"  ✗ {reason}")
    print("\n  Add a valid [options] section to run/config/build.toml:")
    print("")
    print("    [options]")
    print("    profile = \"debug\"    # \"debug\" or \"release\"")
    print("")
    print("    [features]")
    print("    debug-logging = true  # set false to omit the --features flag")
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

PROFILE  = load_options(_CFG)
FEATURES = load_features(_CFG)

# Path to the kernel ELF *inside the container*
# (target/ lives in a docker volume — it is not visible on the host)
# The profile ("debug" / "release") selects the cargo output subdirectory.
KERNEL_ELF_CONTAINER = f"/ferrite_os/target/x86_64-unknown-none/{PROFILE}/kernel"

def tracked_sources() -> list:
    """Source files whose contents decide whether a rebuild is needed."""
    patterns = ("*.rs", "*.toml", "*.ld", "*.conf", "*.cfg")
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
        print("  run/config/build.toml instead of the global PATH:")
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
        print("  Place code.fd and vars.fd in run/deps/ovmf/")
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
    print(f"    profile  = {PROFILE}")
    print(f"    features = {FEATURES}")

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

    # 2. Create ISO directory structure inside the container
    run_in_container(
        "mkdir -p /ferrite_os/build/iso/boot/limine && "
        "mkdir -p /ferrite_os/build/iso/EFI/BOOT"
    )

    # 3. Copy kernel ELF into ISO root
    run_in_container(
        f"cp {KERNEL_ELF_CONTAINER} /ferrite_os/build/iso/ferrite"
    )

    # 4. Copy Limine config (strip CRLF so Limine can parse it)
    run_in_container(
        "sed 's/\\r//' /ferrite_os/run/config/limine.conf "
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

def clean():
    banner("Cleaning")
    ensure_container_running()
    run_in_container("rm -rf /ferrite_os/target/* /ferrite_os/build/*")
    if CACHE.exists():
        CACHE.unlink()
        print(f"  ✓ Cache gelöscht")
    print("  ✓ Clean done")

# ─── terminal helpers ──────────────────────────────────────────────────────────

def _save_terminal() -> tuple:
    """
    Snapshot terminal state before raw serial data can corrupt it.
    Returns (stty_state, terminal_size) — either may be None if unavailable.
    """
    saved_stty = None
    saved_size = None

    if sys.stdout.isatty():
        try:
            saved_size = os.get_terminal_size(sys.stdout.fileno())
        except OSError:
            pass

    if sys.platform != "win32" and sys.stdin.isatty():
        try:
            saved_stty = subprocess.check_output(
                ["stty", "-g"], stdin=sys.stdin, stderr=subprocess.DEVNULL
            ).decode().strip()
        except Exception:
            pass

    return saved_stty, saved_size


def _restore_terminal(saved_stty: str | None, saved_size):
    """
    Restore terminal size and mode, then wipe the screen including scrollback.
    """
    # Restore window size via xterm CSI sequence (harmless if unsupported)
    if saved_size:
        sys.stdout.write(f"\033[8;{saved_size.lines};{saved_size.columns}t")

    # Soft-reset: scroll region, text attributes, wrap mode, etc.
    sys.stdout.write(
        "\033[!p"   # DECSTR  – soft terminal reset
        "\033[r"    # DECSTBM – reset scroll region to full screen
        "\033[m"    # SGR 0   – reset all text attributes
        "\033[?7h"  # DECAWM  – re-enable auto-wrap
    )

    # Clear visible screen + scrollback buffer so no broken serial output remains
    sys.stdout.write(
        "\033[H"    # cursor to row 1, col 1
        "\033[2J"   # erase visible screen
        "\033[3J"   # erase scrollback buffer
    )
    sys.stdout.flush()

    # Restore the full line-discipline snapshot (baud, echo, raw mode, …)
    if saved_stty and sys.platform != "win32":
        try:
            subprocess.run(
                ["stty", saved_stty],
                stdin=sys.stdin,
                stderr=subprocess.DEVNULL,
            )
        except Exception:
            pass

# ─── serial streaming ──────────────────────────────────────────────────────────

def stream_serial(qemu_proc):
    saved_stty, saved_size = _save_terminal()

    # Ensure the log directory exists and open the log file
    LOG.parent.mkdir(parents=True, exist_ok=True)
    log_file = LOG.open("wb")
    print(f"  Logging serial output to {LOG}")

    print(f"  Waiting for serial on {HOST}:{TCP_SERIAL_PORT}...")
    while True:
        try:
            sock = socket.create_connection((HOST, TCP_SERIAL_PORT), timeout=1)
            sock.settimeout(None)
            break
        except (ConnectionRefusedError, TimeoutError, OSError):
            if qemu_proc.poll() is not None:
                print("  ✗ QEMU exited before serial port opened")
                log_file.close()
                _restore_terminal(saved_stty, saved_size)
                return
            time.sleep(RETRY_DELAY)

    banner(f"Serial Output  [{HOST}:{TCP_SERIAL_PORT}]")
    try:
        with sock:
            buf     = b""
            started = False
            while True:
                data = sock.recv(4096)
                if not data:
                    break

                if not started:
                    # Buffer until the kernel's first log line appears;
                    # everything before it is UEFI/firmware noise.
                    buf += data
                    idx = buf.find(SERIAL_START_MARKER)
                    if idx != -1:
                        started = True
                        out = buf[idx:]      # drop all pre-marker bytes
                        sys.stdout.buffer.write(out)
                        sys.stdout.buffer.flush()
                        log_file.write(out)
                        log_file.flush()
                        buf = b""            # free the pre-marker buffer
                    elif len(buf) > 65536:
                        # Don't grow unboundedly — keep a tail in case the
                        # marker is split across two recv() calls
                        buf = buf[-len(SERIAL_START_MARKER):]
                else:
                    sys.stdout.buffer.write(data)
                    sys.stdout.buffer.flush()
                    log_file.write(data)
                    log_file.flush()
    except (KeyboardInterrupt, ConnectionResetError):
        pass
    finally:
        log_file.close()
        _restore_terminal(saved_stty, saved_size)

    print("\n  Serial connection closed")
    print(f"  Log saved to {LOG}")

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
