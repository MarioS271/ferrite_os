#!/usr/bin/env bash
# Starts the FerriteOS ISO in QEMU using the OVMF files from ./ovmf.
# Usage: run.sh [path/to/ferrite_os.iso]
# Default: ferrite_os-release.iso, then the first ferrite_os-*.iso next to this script,
# then <repo>/build/ferrite_os.iso.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OVMF_DIR="$DIR/ovmf"

find_iso() {
    local iso
    for iso in "$DIR/ferrite_os-release.iso" "$DIR"/ferrite_os-*.iso "$DIR/ferrite_os.iso"; do
        [ -f "$iso" ] && { echo "$iso"; return; }
    done
    echo "$DIR/../../../build/ferrite_os.iso"
}

if [ $# -ge 1 ]; then
    ISO="$1"
else
    ISO="$(find_iso)"
fi

[ -f "$ISO" ] || { echo "✗ ISO not found: $ISO" >&2; exit 1; }
command -v qemu-system-x86_64 >/dev/null || { echo "✗ QEMU missing, run install-deps.sh" >&2; exit 1; }
[ -f "$OVMF_DIR/code.fd" ] && [ -f "$OVMF_DIR/vars.fd" ] || { echo "✗ OVMF missing, run install-deps.sh" >&2; exit 1; }

# Work on a copy of vars.fd so the pristine template stays untouched.
VARS="$(mktemp)"
trap 'rm -f "$VARS"' EXIT
cp "$OVMF_DIR/vars.fd" "$VARS"

qemu-system-x86_64 \
    -cdrom "$ISO" \
    -m 4G \
    -vga std \
    -serial stdio \
    -drive "if=pflash,format=raw,readonly=on,file=${OVMF_DIR//,/,,}/code.fd" \
    -drive "if=pflash,format=raw,file=${VARS//,/,,}"
