-- Config for the end-to-end test (testing/vm/e2e.sh). Every binding touches
-- a marker file named after it, which press.py checks after each combo.

local function mark(name)
	return "touch /tmp/wbindkeys-e2e/" .. name
end

-- Physical keys for & and _ on French AZERTY and US QWERTY
bind("MOD+8", mark("mod-8"))
bind("MOD+1", mark("mod-1"))
bind("MOD+SHIFT+7", mark("mod-shift-7"))
bind("MOD+SHIFT+DASH", mark("mod-shift-dash"))

-- ALT matches either side, RIGHTALT only the right
bind("ALT+E", mark("alt-e"))
bind("RIGHTALT+T", mark("rightalt-t"))

-- When both match, the one-sided binding wins
bind("ALT+K", mark("alt-k"))
bind("RIGHTALT+K", mark("rightalt-k"))

-- Modifiers are pressed in the order they are written
bind("CTRL+ALT+T", mark("ctrl-alt-t"))

-- Combos that are only modifiers
bind("RIGHTCTRL", mark("rightctrl"))
bind("SHIFT+ALT", mark("shift-alt"))
