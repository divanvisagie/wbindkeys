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

wbindkeys needs the libinput, libudev, libevdev and libxkbcommon development packages to build. On Debian/Ubuntu:

```sh
sudo apt-get install pkg-config libevdev-dev libudev-dev libinput-dev libxkbcommon-dev
```

On other distributions install the equivalent packages with your package manager.

### From crates.io

```sh
cargo install wbindkeys
sudo ~/.cargo/bin/wbindkeys permissions --set
wbindkeys man --install    # optional, so `man wbindkeys` works
```

Then create a [config](#config) and run `wbindkeys`. To start it automatically when you log in, install it as a service:

```sh
wbindkeys service --install
```

This detects your system's service manager and sets up a per-user service that runs the `wbindkeys` binary you invoked it as. For now only systemd on Linux is supported, where it writes `~/.config/systemd/user/wbindkeys.service`. On anything else it tells you the system isn't supported yet; please [open an issue](https://github.com/divanvisagie/wbindkeys/issues) to ask for it. `wbindkeys service --uninstall` removes the service again.

### From source

```sh
make builddep       # installs Rust (if needed) and the packages above, Debian/Ubuntu only
make permissions    # grants access to input devices, see Permissions
make install
```

`make install` builds the release binary, copies it to `~/.local/bin` (with the man page in `~/.local/share/man/man1`), and runs `wbindkeys service --install` to install and start a per-user service that runs it.

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

A binding runs as soon as its combo is pressed. Pass `{ on = "release" }` to run it once every key of the combo has been let go instead. It doesn't run if another key is pressed in between:

```lua
-- Send Super+1 with ydotool. On release, so the Alt you're holding isn't
-- added to it and turned into Alt+Super+1.
bind("ALT+1", "ydotool key 125:1 2:1 2:0 125:0", { on = "release" })
-- Open the launcher by tapping Super on its own, without running it for
-- every Super+key combo
bind("MOD", "wofi --show drun", { on = "release" })
```

After changing the config, restart the service: `systemctl --user restart wbindkeys.service`.

### Key names

Key names are case-insensitive and joined with `+`. They follow your keyboard layout: a binding means the keys you see labelled with it, so on a French AZERTY keyboard `ALT+A` is the key labelled A, and `MOD+&` and `MOD+_` are the keys labelled `&` and `_`.

| Kind | Names |
|---|---|
| Characters | Any character your layout can type: `A`–`Z`, `0`–`9`, `&`, `_`, `@`, `é`, … Use `PLUS` for `+`, since it separates keys |
| Modifiers | `ALT`, `CTRL`, `SHIFT`, `MOD` (the Super/Windows key), `ALTGR`, `SPACE` |
| One-sided modifiers | `LEFTALT`, `LEFTCTRL`, `LEFTSHIFT`, `LEFTMOD`, `RIGHTALT`, `RIGHTCTRL`, `RIGHTSHIFT`, `RIGHTMOD` |
| Function keys | `F1`–`F12` |
| Navigation | `UP`, `DOWN`, `LEFT`, `RIGHT`, `HOME`, `END`, `PAGEUP`, `PAGEDOWN`, `INSERT`, `DELETE` |
| Other keys | `ESCAPE`, `ENTER`, `BACKSPACE`, `TAB`, `BACKTICK`, `COMMA`, `PERIOD`, `SLASH`, `BACKSLASH`, `SEMICOLON`, `QUOTE`, `DASH`, `EQUAL`, `LEFTSQUARE`, `RIGHTSQUARE`, and any [xkb keysym name](https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h), like `XF86AudioMute` |
| Mouse | `MOUSE1`–`MOUSE10`, `SCROLLUP`, `SCROLLDOWN`, `SCROLLLEFT`, `SCROLLRIGHT` |

A character that needs Shift or AltGr adds it to the binding: on a US keyboard `MOD+&` is Super+Shift+7, and on French `MOD+@` is Super+AltGr+0. With several layouts, characters are looked up in the first one, then the others.

`ALT`, `CTRL`, `SHIFT`, `MOD` and `ALTGR` match every key that acts as that modifier, on either side of the keyboard, including keys your layout options change, like Caps Lock set to act as Ctrl; use the `LEFT…` and `RIGHT…` names to bind only one side. If a combo matches more than one binding, the one-sided binding wins, so with both `ALT+E` and `RIGHTALT+E` bound, Right Alt+E runs the `RIGHTALT+E` command. Modifiers can be pressed in any order. A key name your layout doesn't have is an error when wbindkeys starts.

#### Keyboard layout

wbindkeys asks your compositor for the layout you're using, so it matches what you type. To choose one yourself, for example when there is no compositor, set it in your config with the layout names `localectl` uses:

```lua
keyboard{ layout = "fr" }
keyboard{ layout = "us,se", options = "caps:swapescape" }
```

`keyboard` takes `layout`, `variant` and `options`. Without it and without a compositor, wbindkeys uses `us`. Changing layouts takes effect when wbindkeys restarts.

## Command line

```
wbindkeys                              run wbindkeys (this is what the service does)
wbindkeys --debug                      run with log output, see "Debugging your config"
wbindkeys permissions --check          check wbindkeys can read your input devices
sudo wbindkeys permissions --set       install the udev rule that grants that access
wbindkeys service --install            start wbindkeys when you log in
wbindkeys service --uninstall          stop that and remove the service
wbindkeys record                       record key presses for a bug report, see below
wbindkeys man                          print the wbindkeys(1) man page
wbindkeys man --install                install it next to the binary, see below
wbindkeys --help                       show all options
wbindkeys --version                    show the version
```

`cargo install` only installs the binary, so the man page is built into it instead. `wbindkeys man --install` writes it to `../share/man/man1/` relative to the binary, i.e. `~/.cargo/share/man/man1/wbindkeys.1`. `man` searches there automatically for anything in `~/.cargo/bin` on your `PATH`, so `man wbindkeys` works with no `MANPATH` changes. `make install` does the same for `~/.local/bin`. The page's source is [`man/wbindkeys.1`](man/wbindkeys.1), written by hand in mdoc; a test checks it mentions every subcommand and flag.

## Debugging your config

Run with `--debug` (or `-d`) to print log output to stderr: which input devices were opened, the config path being loaded, the keyboard layout, each registered binding and the key codes it maps to, every key press with its combo, and whether a combo matched a binding.

```sh
systemctl --user stop wbindkeys.service
wbindkeys --debug
```

If the output shows `Failed to open input device ... Permission denied`, or key presses don't show up at all, see [Permissions](#permissions).

## Reporting a key combination that doesn't work

If a binding doesn't fire, record what wbindkeys sees while you press it:

```sh
wbindkeys record
```

Press the combination, then Ctrl+C. This saves `wbindkeys-recording.txt` (use `-o` to pick another file), with each key press, what wbindkeys made of it and which binding it would run, plus your wbindkeys version, system, keyboard layout, input devices and bindings (but not their commands). [Open an issue](https://github.com/divanvisagie/wbindkeys/issues) and attach the file. Everything you type while recording is saved, so don't type passwords.

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

`make e2e` runs an end-to-end test without touching your session: it starts wbindkeys in an LXD virtual machine (created on the first run), presses key combos on a virtual keyboard there and checks which bindings fire. It needs [LXD](https://canonical.com/lxd) with virtual machine support; see `testing/vm/e2e.sh` for options.
