# wbindkeys

[![Crates.io](https://img.shields.io/crates/v/wbindkeys.svg)](https://crates.io/crates/wbindkeys)

A Wayland replacement for `xbindkeys`.

The way that Wayland works these days means that in general your keybinds are now attached to the desktop environment that you are using.
This breaks the Unix philosophy and does not allow you to move a keybind config across desktop environments with ease like you used to be
able to with xbindkeys.

wbindkeys allows you to bind keys, key combos, mouse buttons and scroll events to shell commands, all configured in Lua.

## Philosophy

While wbindkeys intends to replace xbindkeys, in the spirit of Wayland being a replacement with a better API, wbindkeys will offer a new
configuration file language that is easier to handle for both machines and humans alike.

wbindkeys uses Lua for maximum configurability, because sometimes you need an if statement in your config.

wbindkeys reads input directly from your input devices through libinput rather than through the compositor, so it works the same under any Wayland compositor. This is also why it needs permission to read `/dev/input` (see [Permissions](#permissions)).

## Installation and setup

wbindkeys needs the libinput, libudev and libevdev development packages to build. On Debian/Ubuntu:

```sh
sudo apt-get install pkg-config libevdev-dev libudev-dev libinput-dev
```

On other distributions install the equivalent packages with your package manager.

### From crates.io

```sh
cargo install wbindkeys
sudo ~/.cargo/bin/wbindkeys permissions --set
```

Then create a [config](#config) and run `wbindkeys`. To start it automatically when you log in, add a systemd user service at `~/.config/systemd/user/wbindkeys.service`:

```ini
[Unit]
Description=wbindkeys service

[Service]
Type=simple
ExecStart=%h/.cargo/bin/wbindkeys

[Install]
WantedBy=default.target
```

and enable it with `systemctl --user enable --now wbindkeys.service`.

### From source

```sh
make builddep       # installs Rust (if needed) and the packages above, Debian/Ubuntu only
make permissions    # grants access to input devices, see Permissions
make install
```

`make install` builds the release binary, copies it to `~/.local/bin`, and installs + starts a systemd user service (`wbindkeys.service`) that runs it.

## Permissions

wbindkeys reads your keyboard and mouse directly from `/dev/input`, which normal users can't read by default. It runs as your user rather than as root, so it needs to be granted access once per machine.

### Granting access

```sh
sudo wbindkeys permissions --set
```

This installs a udev rule at `/etc/udev/rules.d/69-wbindkeys.rules` that tags input devices with `uaccess`, then re-applies it to devices that are already connected. With that tag, systemd-logind gives whoever is logged in at the machine (the active local session) read access to input devices, and should move that access along when you switch users. Nothing is granted to users logged in remotely or to other accounts.

`sudo` often can't find binaries in `~/.cargo/bin` or `~/.local/bin`, so you may need the full path: `sudo ~/.cargo/bin/wbindkeys permissions --set` or `sudo ~/.local/bin/wbindkeys permissions --set`. When building from source, `make permissions` does this for you.

### Checking access

```sh
wbindkeys permissions --check
```

Run this as your normal user, not with `sudo`, since that is who wbindkeys runs as. It reports whether the udev rule is installed and lists any input devices you can't read:

```
[ok]   udev rule installed at /etc/udev/rules.d/69-wbindkeys.rules
[fail] 17 of 18 input devices are not readable:
         /dev/input/event0 (Power Button): Permission denied (os error 13)
         ...
```

It exits with a non-zero status if anything is wrong, so it can also be used in scripts.

### If access is still denied

- **Re-run `sudo wbindkeys permissions --set`.** Access is granted when a device is added, so devices that were connected while another user (or the login screen) was active can end up belonging to that user. Re-running re-applies access for the current session.
- **Log out and back in**, or reboot, if re-running doesn't help.
- **Make sure you're in a local session.** Access only goes to the session at the physical seat, so it won't work over SSH, or from a session that isn't the active one.
- As a last resort you can add yourself to the `input` group (`sudo usermod -aG input $USER`, then log out and back in). This works regardless of session, but lets every program you run read your keyboard, so the udev rule is preferred.

## Config

wbindkeys loads its config from `$XDG_CONFIG_HOME/wbindkeys/init.lua`, which is `~/.config/wbindkeys/init.lua` unless you have set `$XDG_CONFIG_HOME`. The file must exist, otherwise wbindkeys exits with an error on startup.

Each binding maps a key combo to a shell command:

```lua
-- Run Alacritty on ALT+A
bind("ALT+A", "alacritty")
-- Run Telegram on ALT+T
bind("ALT+T", "flatpak run org.telegram.desktop")
-- Mouse buttons and the scroll wheel can be bound too
bind("MOD+Mouse4", "wofi --show drun")
bind("ALT+ScrollUp", "pactl set-sink-volume @DEFAULT_SINK@ +5%")
```

After changing the config, restart the service: `systemctl --user restart wbindkeys.service`.

### Key names

Key names are case-insensitive and joined with `+`.

| Kind | Names |
|---|---|
| Modifiers | `ALT`, `CTRL`, `SHIFT`, `MOD` (the Super/Windows key), `SPACE` |
| Right-hand modifiers | `RIGHTALT`, `RIGHTCTRL`, `RIGHTSHIFT`, `RIGHTMOD` |
| Letters and numbers | `A`–`Z`, `0`–`9` |
| Function keys | `F1`–`F12` |
| Navigation | `UP`, `DOWN`, `LEFT`, `RIGHT`, `HOME`, `END`, `PAGEUP`, `PAGEDOWN`, `INSERT`, `DELETE` |
| Other keys | `ESCAPE`, `ENTER`, `BACKSPACE`, `TAB`, `BACKTICK`, `COMMA`, `PERIOD`, `SLASH`, `BACKSLASH`, `SEMICOLON`, `QUOTE`, `DASH`, `EQUAL`, `LEFTSQUARE`, `RIGHTSQUARE` |
| Mouse | `MOUSE1`–`MOUSE10`, `SCROLLUP`, `SCROLLDOWN`, `SCROLLLEFT`, `SCROLLRIGHT` |

`ALT`, `CTRL`, `SHIFT` and `MOD` refer to the left-hand keys; use the `RIGHT…` names to bind the right-hand ones. When a binding has several modifiers, press them in the order they are written (`CTRL+ALT+T` means Ctrl, then Alt, then T). Unknown key names are ignored; run with `--debug` to see them.

## Command line

```
wbindkeys                              run wbindkeys (this is what the service does)
wbindkeys --debug                      run with log output, see "Debugging your config"
wbindkeys permissions --check          check wbindkeys can read your input devices
sudo wbindkeys permissions --set       install the udev rule that grants that access
wbindkeys --help                       show all options
wbindkeys --version                    show the version
```

## Debugging your config

Run with `--debug` (or `-d`) to print log output to stderr: which input devices were opened, the config path being loaded, each registered binding and the key codes it maps to, any unrecognised key names, every key press with its combo, and whether a combo matched a binding.

```sh
systemctl --user stop wbindkeys.service
wbindkeys --debug
```

If the output shows `Failed to open input device ... Permission denied`, or key presses don't show up at all, see [Permissions](#permissions).

## Development setup

```sh
make builddep
make permissions    # once, so you can read input devices without sudo
make build-debug    # or `make build-release`; run `make` to list all targets
cargo test
```

To try your changes:

- `make run` builds and runs wbindkeys against your normal config.
- `make test-run` runs it against `testing/config/wbindkeys/init.lua`, where every binding appends a line to `testing/wbindkeys-test.log` instead of launching an app. Watch it with `tail -f testing/wbindkeys-test.log` and press the combos from that file.

Stop the service first (`systemctl --user stop wbindkeys.service`) so the installed copy doesn't also react to your key presses.
