"""
File:
    view-serial-win.py

Authors:
    MarioS271

Copyright:
    SPDX-License-Identifier: GPL-3.0-only

Description:
    Watches the ferrite_os QEMU serial TCP port and connects via ncat
    once QEMU is up. Port must match TCP_SERIAL_PORT in build-win.py.

Usage:
    python view-serial-win.py
"""

import subprocess
import shutil
import sys
import time
import socket

# ─── config ───────────────────────────────────────────────────────────────────

HOST            = "localhost"
TCP_SERIAL_PORT = 4231        # must match TCP_SERIAL_PORT in build-win.py
RETRY_DELAY     = 1.0         # seconds between availability checks

# ─── helpers ──────────────────────────────────────────────────────────────────

def banner(msg: str):
    print(f"\n{'─'*50}\n  {msg}\n{'─'*50}")

def check_ncat():
    if shutil.which("ncat") is None:
        print("  ✗ ncat not found in PATH")
        print("    Install it with:  winget install nmap")
        sys.exit(1)
    print("  ✓ ncat found")

def wait_for_port():
    print(f"  Waiting for QEMU on {HOST}:{TCP_SERIAL_PORT}...")
    while True:
        try:
            with socket.create_connection((HOST, TCP_SERIAL_PORT), timeout=1):
                print(f"  ✓ Port {TCP_SERIAL_PORT} is open")
                return
        except (ConnectionRefusedError, TimeoutError, OSError):
            time.sleep(RETRY_DELAY)

def connect():
    banner(f"Serial Output  [{HOST}:{TCP_SERIAL_PORT}]")
    subprocess.run(["ncat", HOST, str(TCP_SERIAL_PORT)])

# ─── main ─────────────────────────────────────────────────────────────────────

def main():
    sys.stdout.reconfigure(encoding="utf-8")

    banner("ferrite_os Serial Viewer")
    check_ncat()

    while True:
        wait_for_port()
        connect()
        print("  ✗ ncat disconnected — waiting for QEMU...")

if __name__ == "__main__":
    main()