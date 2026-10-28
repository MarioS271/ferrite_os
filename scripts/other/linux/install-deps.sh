#!/usr/bin/env bash
# Installs everything needed to run the FerriteOS ISO:
#   - qemu-system-x86_64 (system-wide, via the distro package manager)
#   - OVMF (code.fd / vars.fd), downloaded next to this script into ./ovmf
set -euo pipefail

# Double-click support: when not started from a terminal, reopen in one and
# keep it open at the end so the output stays readable.
if [ -z "${FERRITE_TERMINAL:-}" ] && [ ! -t 1 ]; then
    export FERRITE_TERMINAL=1
    self="$(readlink -f "${BASH_SOURCE[0]}")"
    for term in x-terminal-emulator gnome-terminal konsole xfce4-terminal kitty alacritty xterm; do
        command -v "$term" >/dev/null 2>&1 || continue
        case "$term" in
            gnome-terminal) exec "$term" -- bash "$self" "$@" ;;
            xfce4-terminal) exec "$term" -x bash "$self" "$@" ;;
            kitty)          exec "$term" bash "$self" "$@" ;;
            *)              exec "$term" -e bash "$self" "$@" ;;
        esac
    done
fi

on_exit() {
    local code=$?
    [ -n "${tmp:-}" ] && rm -rf "$tmp"
    [ -n "${VARS:-}" ] && rm -f "$VARS"
    if [ -n "${FERRITE_TERMINAL:-}" ]; then
        read -rp "Press Enter to close..." _ || true
    fi
    exit "$code"
}
trap on_exit EXIT

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OVMF_DIR="$DIR/ovmf"
OVMF_API="https://api.github.com/repos/rust-osdev/ovmf-prebuilt/releases/latest"

SUDO=""
[ "$(id -u)" -ne 0 ] && SUDO="sudo"

install_qemu() {
    if command -v qemu-system-x86_64 >/dev/null 2>&1; then
        echo "✓ qemu-system-x86_64 already installed"
        return
    fi
    echo ">> Installing QEMU system-wide"
    if   command -v apt-get >/dev/null; then $SUDO apt-get update && $SUDO apt-get install -y qemu-system-x86
    elif command -v dnf     >/dev/null; then $SUDO dnf install -y qemu-system-x86
    elif command -v pacman  >/dev/null; then $SUDO pacman -S --needed --noconfirm qemu-system-x86
    elif command -v zypper  >/dev/null; then $SUDO zypper install -y qemu-x86
    elif command -v apk     >/dev/null; then $SUDO apk add qemu-system-x86_64
    else
        echo "✗ No supported package manager found, install qemu-system-x86_64 manually" >&2
        exit 1
    fi
}

install_ovmf() {
    if [ -f "$OVMF_DIR/code.fd" ] && [ -f "$OVMF_DIR/vars.fd" ]; then
        echo "✓ OVMF already present in $OVMF_DIR"
        return
    fi
    for tool in curl tar xz; do
        command -v "$tool" >/dev/null || { echo "✗ '$tool' is required" >&2; exit 1; }
    done

    echo ">> Downloading OVMF (rust-osdev/ovmf-prebuilt)"
    url="$(curl -fsSL "$OVMF_API" | grep -o '"browser_download_url": *"[^"]*-bin\.tar\.xz"' | head -n1 | cut -d'"' -f4)"
    [ -n "$url" ] || { echo "✗ Could not find OVMF release asset" >&2; exit 1; }

    tmp="$(mktemp -d)"
    curl -fsSL "$url" | tar -xJ -C "$tmp"

    mkdir -p "$OVMF_DIR"
    cp "$tmp"/*/x64/code.fd "$tmp"/*/x64/vars.fd "$OVMF_DIR/"
    echo "✓ OVMF installed to $OVMF_DIR"
}

install_qemu
install_ovmf
echo "Done. Start the ISO with: $DIR/run.sh [path/to/ferrite_os.iso]"
