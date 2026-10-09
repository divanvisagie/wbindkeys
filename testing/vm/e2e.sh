#!/bin/bash
# End-to-end test: runs wbindkeys in an LXD virtual machine and presses real
# key combos on a uinput virtual keyboard there, checking which bindings fire.
#
# It's a VM rather than a container because input devices belong to the
# kernel: a virtual keyboard made in a container would type into the host's
# desktop session.
#
# Usage: testing/vm/e2e.sh [path to wbindkeys binary]
#
#   E2E_VM     name of the VM (default wbindkeys-e2e), created on first run
#   E2E_IMAGE  image to create it from (default ubuntu:26.04); the binary is
#              built on the host, so it needs a glibc at least as new
#   E2E_KEEP=1 leave the VM running afterwards, e.g. to poke around with
#              `lxc exec wbindkeys-e2e -- bash`
#
# Delete the VM with `lxc delete --force wbindkeys-e2e`.

set -euo pipefail

VM="${E2E_VM:-wbindkeys-e2e}"
IMAGE="${E2E_IMAGE:-ubuntu:26.04}"
BIN="${1:-target/release/wbindkeys}"
HERE="$(cd "$(dirname "$0")" && pwd)"
GUEST_DIR=/root/e2e

if [ ! -x "$BIN" ]; then
    echo "error: $BIN not found, build it first (make build-release)"
    exit 1
fi

if ! lxc info "$VM" >/dev/null 2>&1; then
    echo "Creating VM $VM from $IMAGE (the first run downloads the image)..."
    lxc launch --quiet "$IMAGE" "$VM" --vm
elif [ "$(lxc list "$VM" --columns s --format csv)" != RUNNING ]; then
    lxc start "$VM"
fi

if [ "${E2E_KEEP:-}" != 1 ]; then
    trap 'lxc stop "$VM" >/dev/null 2>&1 || true' EXIT
fi

echo "Waiting for $VM to boot..."
for _ in $(seq 180); do
    lxc exec "$VM" -- true >/dev/null 2>&1 && break
    sleep 1
done
lxc exec "$VM" -- cloud-init status --wait >/dev/null 2>&1 || true

lxc exec "$VM" -- sh -c '
    packages="libinput10 libxkbcommon0 xkb-data python3-evdev"
    dpkg -s $packages >/dev/null 2>&1 && exit 0
    echo "Installing $packages in the VM..."
    apt-get update -q >/dev/null &&
        DEBIAN_FRONTEND=noninteractive apt-get install -y -q $packages >/dev/null
'

lxc exec "$VM" -- rm -rf "$GUEST_DIR"
lxc exec "$VM" -- mkdir -p "$GUEST_DIR"
lxc file push --quiet "$BIN" "$VM$GUEST_DIR/wbindkeys"
lxc file push --quiet "$HERE/press.py" "$VM$GUEST_DIR/press.py"
lxc file push --quiet --recursive "$HERE/us" "$HERE/fr" "$VM$GUEST_DIR/"

echo "Running the end-to-end test in $VM..."
status=0
lxc exec "$VM" -- python3 "$GUEST_DIR/press.py" "$GUEST_DIR/wbindkeys" || status=$?
if [ "$status" != 0 ]; then
    lxc file pull --quiet "$VM$GUEST_DIR/wbindkeys.log" "$HERE/wbindkeys.log" 2>/dev/null &&
        echo "Copied wbindkeys --debug output to $HERE/wbindkeys.log"
fi
exit "$status"
