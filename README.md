# wbindkeys
A wayland replacement for xbindkeys

🚧 **This project is currently under construction** 🚧 

You can basically only bind functions to Alt+A and Alt+B and even then not many userspace applications will work because we still need to sort out user privelege and bringing in your variables since it runs as root... for now.

I have really only been able to get it to launch xterm bound to ALT+A.

## Philosophy 

While wbindkeys intends to replace xbindkeys, in the spirit of wayland being a replacement with a better API, wbindkeys will offer a new config file format that is easier to handle for both machines and humans alike.

wbindkeys uses lua for maximum configurability, because sometimes you need an if statement in your config.

### Config Example
```lua
-- Run alacritty on ALT+T
bind("ALT+T", "alacritty")

-- Run a lua function when calling 
bind("ALT+L", function()
    print("Hello, World!")
end)

-- Execute an external script 
bind("ALT+X", "sh -c 'echo Hello, World!'")

-- Execute an external script from inside a function
bind("ALT+Y", function()
    os.execute("sh -c 'echo Hello, World!'")
end)

```

# Roadmap
- [x] Hook into wayland keyboard events
- [x] Get a binding to execute a print from a lua binding config
- [x] Execute the command 
- [ ] Implement full range of keymaps
- [ ] Fix for launching app in userspace on the users privelege level


# Development Setup 

```sh
./configure
make
```

The following configuration allows usage without sudo:

```sh
sudo usermod -aG input yourusername
```
Creating a udev rule involves creating a file in /etc/udev/rules.d/ (e.g., 99-input.rules) with contents along the lines of:

```
ACTION=="add", KERNEL=="event*", SUBSYSTEM=="input", MODE="660", GROUP="input"
```
