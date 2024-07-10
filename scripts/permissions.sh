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

# Add the current user to the input group
CURRENT_USER=$(logname)
usermod -aG input "$CURRENT_USER"
echo "Added $CURRENT_USER to the input group."

# Create a udev rule for input devices
UDEV_RULE='/etc/udev/rules.d/99-input.rules'
echo 'ACTION=="add", KERNEL=="event*", SUBSYSTEM=="input", MODE="660", GROUP="input"' > "$UDEV_RULE"
udevadm control --reload-rules && udevadm trigger
echo "Created udev rule at $UDEV_RULE."

