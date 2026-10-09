-- US config for the end-to-end test (testing/vm/e2e.sh). Every binding
-- touches a marker file named after it, which press.py checks after each
-- combo. The VM has no compositor to ask for the layout, so it's set here.
keyboard{ layout = "us" }

local function mark(name)
	return "touch /tmp/wbindkeys-e2e/" .. name
end

bind("MOD+8", mark("mod-8"))
bind("MOD+1", mark("mod-1"))
-- & is Shift+7, so this adds the Shift; _ is written out in full
bind("MOD+&", mark("mod-amp"))
bind("MOD+SHIFT+DASH", mark("mod-shift-dash"))

-- ALT matches either side, RIGHTALT only the right
bind("ALT+E", mark("alt-e"))
bind("RIGHTALT+T", mark("rightalt-t"))

-- When both match, the one-sided binding wins
bind("ALT+K", mark("alt-k"))
bind("RIGHTALT+K", mark("rightalt-k"))

-- Modifiers can be pressed in any order
bind("CTRL+ALT+T", mark("ctrl-alt-t"))

-- Combos that are only modifiers
bind("RIGHTCTRL", mark("rightctrl"))
bind("SHIFT+ALT", mark("shift-alt"))

-- Release bindings run once the combo's keys are let go. Tapping Super on
-- its own runs MOD, but the Super+key cases above must not.
bind("MOD", mark("mod-tap"), { on = "release" })
bind("ALT+1", mark("alt-1-release"), { on = "release" })
