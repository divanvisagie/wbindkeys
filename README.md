# wbindkeys
A wayland replacement for xbindkeys

🚧 **This project is currently under construction** 🚧 

You can bind direct bash commands to some key combos but not all combos are supported or tested. Performance is also untested so you may find 
tasks like app switching to be a little unsatisfactory.

## Philosophy 

While wbindkeys intends to replace xbindkeys, in the spirit of wayland being a replacement with a better API, wbindkeys will offer a new config file format that is easier to handle for both machines and humans alike.

wbindkeys uses lua for maximum configurability, because sometimes you need an if statement in your config.

wbindkeys reads input directly from your input devices through libinput rather than through the compositor, so it works the same under any Wayland compositor. This is also why it needs permission to read `/dev/input` (see below).

## Installation and setup

### Install 

Currently the only way to install wbindkeys is to build from source. `make builddep` installs build dependencies with `apt`, so on distributions other than Debian/Ubuntu install Rust, `pkg-config` and the development packages for libevdev, libudev and libinput yourself.

First install build dependencies and grant wbindkeys permission to read input devices (this needs root, and must happen before the service is started so it can access `/dev/input` as your user). `make permissions` builds wbindkeys and runs `sudo wbindkeys permissions --set`:

```sh
make builddep
make permissions
```

Then build and install. `make install` builds the release binary, copies it to `~/.local/bin`, and installs + starts a systemd user service (`wbindkeys.service`) that runs it.

```sh
make install
```

### Config

wbindkeys loads its config from `$XDG_CONFIG_HOME/wbindkeys/init.lua`, which is `~/.config/wbindkeys/init.lua` unless you have set `$XDG_CONFIG_HOME`. The file must exist, otherwise wbindkeys exits with an error on startup.

Each binding maps a key combo to a shell command:

```lua
-- Run alacritty on ALT+A
bind("ALT+A", "alacritty")
-- Run Telegram on ALT+T
bind("ALT+T", "flatpak run org.telegram.desktop")
-- Mouse buttons and the scroll wheel can be bound too
bind("MOD+Mouse4", "wofi --show drun")
bind("ALT+ScrollUp", "pactl set-sink-volume @DEFAULT_SINK@ +5%")
```

After changing the config, restart the service: `systemctl --user restart wbindkeys.service`.

#### Key names

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

### Command line

```
wbindkeys                              run wbindkeys (this is what the service does)
wbindkeys --debug                      run with log output, see "Debugging your config"
wbindkeys permissions --check          check wbindkeys can read your input devices
sudo wbindkeys permissions --set       install the udev rule that grants that access
wbindkeys --help                       show all options
wbindkeys --version                    show the version
```

### Debugging your config

Run with `--debug` (or `-d`) to print log output to stderr: which input devices were opened, the config path being loaded, each registered binding and the key codes it maps to, any unrecognised key names, every key press with its combo, and whether a combo matched a binding.

```sh
systemctl --user stop wbindkeys.service
wbindkeys --debug
```

If the output shows `Permission denied` for input devices, check and fix access with the `permissions` subcommand:

```sh
wbindkeys permissions --check        # is the udev rule installed and are devices readable?
sudo wbindkeys permissions --set     # install the udev rule and re-apply device access
```

`--check` should be run as your normal user, since that is who wbindkeys runs as. If `sudo` can't find `wbindkeys` (e.g. it is in `~/.local/bin`), use the full path: `sudo ~/.local/bin/wbindkeys permissions --set`.

## Roadmap to 0.1.0
- [x] Hook into input events via libinput
- [x] Get a binding to execute a print from a lua binding config
- [x] Execute the command 
- [x] Fix for launching app in userspace on the users privilege level
- [x] Mouse button and scroll wheel bindings
- [x] `--debug` output and `permissions` command for troubleshooting
- [ ] Implement and test full range of keymaps
- [ ] Debian installer

## Development Setup 

```sh
make builddep
make permissions    # once, so you can read input devices without sudo
make build-debug    # or `make` for a release build
cargo test
```

To try your changes:

- `make run` builds and runs wbindkeys against your normal config.
- `make test-run` runs it against `testing/config/wbindkeys/init.lua`, where every binding appends a line to `testing/wbindkeys-test.log` instead of launching an app. Watch it with `tail -f testing/wbindkeys-test.log` and press the combos from that file.

Stop the service first (`systemctl --user stop wbindkeys.service`) so the installed copy doesn't also react to your key presses.
