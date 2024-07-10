# wbindkeys
A wayland replacement for xbindkeys

🚧 **This project is currently under construction** 🚧 

You can bind direct bash commands to some key combos but not all combos are supported or tested. Performance is also untested so you may find 
tasks like app switching to be a little unsatisfactory.

## Philosophy 

While wbindkeys intends to replace xbindkeys, in the spirit of wayland being a replacement with a better API, wbindkeys will offer a new config file format that is easier to handle for both machines and humans alike.

wbindkeys uses lua for maximum configurability, because sometimes you need an if statement in your config.

### Config Example

Save the following config in a file named `init.lua` inside your configuration directory (typically found using the `$XDG_CONFIG_HOME` environment variable or defaulting to `~/.config/wbindkeys/`).

```lua
-- Run alacritty on ALT+T
bind("ALT+A", "alacritty")
bind("ALT+T", "flatpak run org.telegram.desktop")
```

### Loading the Configuration

The config file is automatically loaded from the `config_dir()/wbindkeys/init.lua`. Make sure the configuration file exists, otherwise the application will panic.

# Roadmap to 0.1.0
- [x] Hook into wayland keyboard events
- [x] Get a binding to execute a print from a lua binding config
- [x] Execute the command 
- [x] Fix for launching app in userspace on the users privilege level
- [ ] Implement and test full range of keymaps
- [ ] Debian installer

# Development Setup 

```sh
./configure
make
```
