# wbindkeys
A wayland replacement for xbindkeys

**This project is currently under construction and is not usable**

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
- [ ] Get a bindint to execute a print from a lua binding config
- [ ] Execute the command 
- [ ] Implement full range of keymaps
