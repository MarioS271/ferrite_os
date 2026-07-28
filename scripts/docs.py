"""
File:
    docs.py

Authors:
    MarioS271

Copyright:
    SPDX-License-Identifier: GPL-3.0-only

Description:
    Build docs for Ferrite
    Docs creation runs inside Docker

Usage:
    python docs.py [build | open | all | clean]
"""

import subprocess
import shutil
import sys
import os
import tomllib
import webbrowser
from pathlib import Path

# ─── config ───────────────────────────────────────────────────────────────────

ROOT   = Path(__file__).parent.parent
BUILD  = ROOT / "build"
TARGET = "x86_64-unknown-none"

# Crate whose docs we care about (must match the [package] name in Cargo.toml)
CRATE  = "kernel"

# Manifest path *inside the container*
MANIFEST_CONTAINER = "src/kernel/Cargo.toml"

# Generated docs *inside the container* (target/ lives in a docker volume,
# so it is NOT visible on the host — the output has to be copied out)
DOCS_DIR_CONTAINER = f"/ferrite_os/target/{TARGET}/doc"

# Generated docs on the host
DOCS_DIR   = ROOT / "docs"
DOCS_INDEX = DOCS_DIR / CRATE / "index.html"

# Staging dir for docker cp, and a record of what rustdoc put into docs/
# (so clean can remove generated files without touching hand-written ones)
DOCS_TMP      = BUILD / ".docs_tmp"
DOCS_MANIFEST = DOCS_DIR / ".rustdoc_manifest"

# Other global vars
CONTAINER_NAME = "ferrite_os"

def load_config() -> dict:
    """
    Load build.toml if present.

    docs.py only reads [extra_paths], which exists so Windows can find docker
    without a global PATH entry. Everywhere else it's optional, so a missing
    file is not an error here — unlike in build.py.
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

# ─── helpers ──────────────────────────────────────────────────────────────────

def banner(msg: str):
    print(f"\n{'─'*50}\n  {msg}\n{'─'*50}")

def patch_path():
    current = os.environ.get("PATH", "")
    for p in EXTRA_PATHS:
        if p not in current and os.path.exists(p):
            os.environ["PATH"] = p + os.pathsep + current

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

def remove(path: Path):
    """Delete a file or directory if it exists."""
    if path.is_dir():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()

def check_dependencies():
    banner("Checking Dependencies")
    deps = ["docker"]
    missing = [d for d in deps if shutil.which(d) is None]
    if missing:
        print("  Missing tools (not in PATH):")
        for m in missing:
            print(f"  ✗ {m}")
        print("\n  On Windows you can add the install dir to")
        print("  run/config/build.toml instead of the global PATH:")
        print("")
        print("    [extra_paths]")
        print("    paths = [\"C:/Program Files/Docker/Docker/resources/bin\"]")
        sys.exit(1)
    for d in deps:
        print(f"  ✓ {d}")

def list_config_vars():
    banner("Config")
    vars = {
        "ROOT":       ROOT,
        "DOCS_DIR":   DOCS_DIR,
        "DOCS_INDEX": DOCS_INDEX,
    }
    for name, value in vars.items():
        exists = "✓" if Path(str(value)).exists() else "✗"
        print(f"  {exists} {name} = {value}")

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

# ─── docs steps ───────────────────────────────────────────────────────────────

def do_docs():
    banner("Building Docs (in container)")

    ensure_container_running()

    run_in_container(
        "cd /ferrite_os && "
        "cargo doc "
        f"--target {TARGET} "
        f"--manifest-path {MANIFEST_CONTAINER} "
        "--no-deps "
        "--document-private-items"
    )

    # target/ is a docker volume — copy the generated docs onto the host.
    # docker cp nests into an existing destination, so stage it first.
    if DOCS_TMP.exists():
        shutil.rmtree(DOCS_TMP)
    BUILD.mkdir(parents=True, exist_ok=True)

    run(["docker", "cp",
         f"{CONTAINER_NAME}:{DOCS_DIR_CONTAINER}",
         str(DOCS_TMP)])

    # Merge into docs/, replacing only the entries rustdoc generated
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    generated = []
    for entry in DOCS_TMP.iterdir():
        dest = DOCS_DIR / entry.name
        remove(dest)
        shutil.move(str(entry), str(dest))
        generated.append(entry.name)

    shutil.rmtree(DOCS_TMP)
    DOCS_MANIFEST.write_text("\n".join(sorted(generated)))

    if not DOCS_INDEX.exists():
        print(f"  ✗ {DOCS_INDEX} not found after build — something went wrong")
        print(f"    (is the crate actually named '{CRATE}'?)")
        sys.exit(1)

    print(f"  ✓ Docs: {DOCS_INDEX}")

def open_docs():
    banner("Opening Docs")
    if not DOCS_INDEX.exists():
        print("  ✗ No docs found. Run build first.")
        sys.exit(1)
    url = DOCS_INDEX.resolve().as_uri()
    print(f"  >> {url}")
    try:
        opened = webbrowser.open(url)
    except webbrowser.Error:
        opened = False
    if not opened:
        print("  ! No browser available — open the URL above manually")

def clean():
    banner("Cleaning Docs")
    if not DOCS_DIR.exists():
        print("  Nothing to clean")
        return
    shutil.rmtree(DOCS_DIR)
    print(f"  ✓ Deleted {DOCS_DIR}")

# ─── commands ─────────────────────────────────────────────────────────────────

def cmd_build():
    check_dependencies()
    do_docs()

def cmd_open():
    open_docs()

def cmd_all():
    check_dependencies()
    do_docs()
    open_docs()

def cmd_clean():
    clean()

COMMANDS = {
    "build": cmd_build,
    "open":  cmd_open,
    "all":   cmd_all,
    "clean": cmd_clean,
}

DEFAULT_COMMAND = "all"

def main():
    sys.stdout.reconfigure(encoding="utf-8")
    patch_path()
    list_config_vars()

    arg = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_COMMAND

    if arg not in COMMANDS:
        print(f"\n  Usage: python docs.py [{' | '.join(COMMANDS)}]")
        print( "  Commands:")
        print( "    build  — generate rustdoc for the kernel crate in Docker")
        print( "    open   — open the generated docs in the browser")
        print(f"    all    — build + open (default)")
        print( "    clean  — delete generated docs")
        sys.exit(1)

    COMMANDS[arg]()

if __name__ == "__main__":
    main()
