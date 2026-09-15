"""
File:
    lib.py

Authors:
    MarioS271

Copyright:
    SPDX-License-Identifier: GPL-3.0-only

Description:
    Shared helpers for the Ferrite build scripts.

    Everything in here is used by both scripts/x86_64/build.py and
    scripts/docs.py: config loading, command execution, container lifecycle
    and the command dispatcher. Arch- or task-specific logic stays in the
    calling script.

Usage:
    Not executable. Imported via bootstrap(), see build.py / docs.py.
"""

import os
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

# ─── config ───────────────────────────────────────────────────────────────────

# scripts/lib.py → scripts/ → repo root
ROOT = Path(__file__).parent.parent

BUILD  = ROOT / "build"
CONFIG = ROOT / "run" / "config" / "build.toml"

CONTAINER_NAME = "ferrite_os"

def load_config() -> dict:
    """
    Load build.toml if present.

    A missing file is not an error here — only build.py requires [options],
    and it validates that itself.
    """
    if not CONFIG.exists():
        return {}
    try:
        with open(CONFIG, "rb") as f:
            return tomllib.load(f)
    except tomllib.TOMLDecodeError as e:
        print(f"  ! Ignoring malformed {CONFIG}: {e}")
        return {}

CFG         = load_config()
EXTRA_PATHS = CFG.get("extra_paths", {}).get("paths", [])

# ─── helpers ──────────────────────────────────────────────────────────────────

def banner(msg: str):
    print(f"\n{'─'*50}\n  {msg}\n{'─'*50}")

def patch_path():
    """Prepend [extra_paths] entries to PATH, so Windows can find docker/qemu."""
    current = os.environ.get("PATH", "")
    for p in EXTRA_PATHS:
        if p not in current and os.path.exists(p):
            os.environ["PATH"] = p + os.pathsep + current

def remove(path: Path):
    """Delete a file or directory if it exists, surviving read-only entries."""
    def handler(func, target, _exc):
        os.chmod(target, 0o600)
        func(target)

    if path.is_dir():
        if sys.version_info >= (3, 12):
            shutil.rmtree(path, onexc=handler)
        else:
            shutil.rmtree(path, onerror=handler)
    elif path.exists():
        path.unlink()

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

def check_dependencies(deps: list):
    """Verify every dependency is on PATH, or explain how to put it there."""
    banner("Checking Dependencies")
    missing = [d for d in deps if shutil.which(d) is None]
    if missing:
        print("  Missing tools (not in PATH):")
        for m in missing:
            print(f"  ✗ {m}")
        print("\n  On Windows you can list the install dirs in")
        print(f"  {CONFIG.relative_to(ROOT).as_posix()} instead of the global PATH:")
        print("")
        print("    [extra_paths]")
        print("    paths = [")
        print("        \"C:/Program Files/qemu\",")
        print("        \"C:/Program Files/Docker/Docker/resources/bin\",")
        print("    ]")
        sys.exit(1)
    for d in deps:
        print(f"  ✓ {d}")

def list_config_vars(paths: dict, extras: dict = None):
    """Print resolved paths with an existence marker, plus plain key/values."""
    banner("Config")
    for name, value in paths.items():
        exists = "✓" if Path(str(value)).exists() else "✗"
        print(f"  {exists} {name} = {value}")
    for name, value in (extras or {}).items():
        print(f"    {name} = {value}")

# ─── container lifecycle ──────────────────────────────────────────────────────

def compose_env() -> dict:
    """
    Environment for `docker compose`.

    On POSIX the bind mount preserves uids, so a root container would leave
    root-owned files in the repo that the host-side scripts then cannot
    overwrite. Passing the caller's uid/gid makes the container write as them.
    Docker Desktop (Windows/macOS) maps ownership itself, so there the vars
    stay unset and docker-compose.yml falls back to root.
    """
    env = os.environ.copy()
    if os.name == "posix":
        env["FERRITE_UID"] = str(os.getuid())
        env["FERRITE_GID"] = str(os.getgid())
    return env

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
    run(["docker", "compose", "up", "-d", "--build"], cwd=ROOT, env=compose_env())

# ─── command dispatch ─────────────────────────────────────────────────────────

def dispatch(script: str, commands: dict, default: str = None):
    """
    Run the command named by argv[1].

    `commands` maps a name to (help text, [steps]); each step is called in
    order. Falls back to `default` when no argument is given.
    """
    arg = sys.argv[1] if len(sys.argv) > 1 else default

    if arg not in commands:
        print(f"\n  Usage: python {script} [{' | '.join(commands)}]")
        print("  Commands:")
        width = max(len(name) for name in commands)
        for name, (help_text, _) in commands.items():
            suffix = "  (default)" if name == default else ""
            print(f"    {name:<{width}}  — {help_text}{suffix}")
        sys.exit(1)

    for step in commands[arg][1]:
        step()
