-- French AZERTY config for the end-to-end test (testing/vm/e2e.sh): keys
-- are looked up in the layout, so these are the keys labelled with them.
keyboard{ layout = "fr" }

local function mark(name)
	return "touch /tmp/wbindkeys-e2e/" .. name
end

bind("MOD+&", mark("mod-amp")) -- the 1 key
bind("MOD+_", mark("mod-underscore")) -- the 8 key
bind("ALT+A", mark("alt-a")) -- where Q is on QWERTY
bind("MOD+8", mark("mod-8")) -- Shift and the 8 key
bind("MOD+@", mark("mod-at")) -- AltGr and the 0 key
