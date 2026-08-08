#!/bin/bash

BIN_DEST="${HOME}/.local/bin/wbindkeys"
SERVICE_DEST="${HOME}/.config/systemd/user/wbindkeys.service"

# Install the wbindkeys binary
# Assuming wbindkeys is already built and located in the current directory
mkdir -p $(dirname $BIN_DEST)
cp target/release/wbindkeys "$BIN_DEST"
chmod +x "$BIN_DEST"
echo "Installed wbindkeys to $BIN_DEST"

# Create a systemd service
mkdir -p "$(dirname $SERVICE_DEST)"
cat <<EOL > $SERVICE_DEST
[Unit]
Description=wbindkeys service

[Service]
Type=simple
ExecStart=$BIN_DEST

[Install]
WantedBy=default.target
EOL

systemctl --user daemon-reload
systemctl --user enable wbindkeys
systemctl --user start wbindkeys
echo "Created and started systemd service."
