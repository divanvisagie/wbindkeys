#!/bin/bash
set -e

BIN_DEST="${HOME}/.local/bin/wbindkeys"

# Stop the service first so the binary isn't busy when we overwrite it below
# (harmless no-op if the service doesn't exist yet on a first install).
systemctl --user stop wbindkeys 2>/dev/null || true

# Install the wbindkeys binary
# Assuming wbindkeys is already built and located in the current directory
mkdir -p "$(dirname "$BIN_DEST")"
cp target/release/wbindkeys "$BIN_DEST"
chmod +x "$BIN_DEST"
echo "Installed wbindkeys to $BIN_DEST"

# Install the man page to ~/.local/share/man/man1, where man finds it for
# binaries in ~/.local/bin without any MANPATH changes
"$BIN_DEST" man --install

# Install, enable and (re)start the per-user service
"$BIN_DEST" service --install
