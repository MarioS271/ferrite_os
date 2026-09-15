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

import shutil
import sys
import webbrowser

import lib
from lib import BUILD, CONTAINER_NAME, ROOT, banner, remove, run, run_in_container

# ─── config ───────────────────────────────────────────────────────────────────

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

# Staging dir for docker cp — it nests into an existing destination, so the
# copy lands here first and replaces docs/ only once it succeeded.
DOCS_TMP = BUILD / ".docs_tmp"

# ─── docs steps ───────────────────────────────────────────────────────────────

def do_docs():
    banner("Building Docs (in container)")

    lib.ensure_container_running()

    run_in_container(
        "cd /ferrite_os && "
        "cargo doc "
        f"--target {TARGET} "
        f"--manifest-path {MANIFEST_CONTAINER} "
        "--no-deps "
        "--document-private-items"
    )

    # target/ is a docker volume — copy the generated docs onto the host
    remove(DOCS_TMP)
    BUILD.mkdir(parents=True, exist_ok=True)
    run(["docker", "cp", f"{CONTAINER_NAME}:{DOCS_DIR_CONTAINER}", str(DOCS_TMP)])

    # docs/ is entirely rustdoc output, so swap it wholesale
    remove(DOCS_DIR)
    shutil.move(str(DOCS_TMP), str(DOCS_DIR))

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
    remove(DOCS_DIR)
    print(f"  ✓ Deleted {DOCS_DIR}")

# ─── commands ─────────────────────────────────────────────────────────────────

def check_deps():
    lib.check_dependencies(["docker"])

COMMANDS = {
    "build": ("generate rustdoc for the kernel crate in Docker", [check_deps, do_docs]),
    "open":  ("open the generated docs in the browser",          [open_docs]),
    "all":   ("build + open",                                    [check_deps, do_docs, open_docs]),
    "clean": ("delete generated docs",                           [clean]),
}

def main():
    sys.stdout.reconfigure(encoding="utf-8")
    lib.patch_path()
    lib.list_config_vars({
        "ROOT":       ROOT,
        "DOCS_DIR":   DOCS_DIR,
        "DOCS_INDEX": DOCS_INDEX,
    })
    lib.dispatch("docs.py", COMMANDS, default="all")

if __name__ == "__main__":
    main()
