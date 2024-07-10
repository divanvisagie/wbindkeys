#!/bin/bash
# Install the wbindkeys binary
# Assuming wbindkeys is already built and located in the current directory
cp target/release/wbindkeys /usr/local/bin/
chmod +x /usr/local/bin/wbindkeys
echo "Installed wbindkeys to /usr/local/bin."

# Create a systemd service
SYSTEMD_SERVICE='/etc/systemd/system/wbindkeys.service'
cat <<EOL > $SYSTEMD_SERVICE
[Unit]
Description=wbindkeys service

[Service]
ExecStart=/usr/local/bin/wbindkeys
User=$CURRENT_USER

[Install]
WantedBy=multi-user.target
EOL

systemctl daemon-reload
systemctl enable wbindkeys
systemctl start wbindkeys
echo "Created and started systemd service."

echo "Installation completed. Please log out and log back in for changes to take effect."
