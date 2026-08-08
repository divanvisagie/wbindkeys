#!/bin/bash

# Function to check if a command exists
command_exists() {
    command -v "$1" &>/dev/null
}

# Ensure the script is run as root
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root."
    exit 1
fi

# Create a udev rule for input devices
UDEV_RULE='/etc/udev/rules.d/69-wbindkeys.rules'
echo 'ACTION=="add", KERNEL=="event*", SUBSYSTEM=="input", TAG+="uaccess", TAG+="seat"' > "$UDEV_RULE"
udevadm control --reload-rules && udevadm trigger
echo "Created udev rule at $UDEV_RULE."
