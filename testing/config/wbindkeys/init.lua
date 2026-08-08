-- Test config for local PR verification.
-- Every binding just appends a timestamped line to testing/wbindkeys-test.log
-- so we can confirm wbindkeys actually fired a shortcut, without launching
-- real applications. Point wbindkeys at this config with:
--
--   make test-run
--
-- then in another terminal:
--
--   tail -f testing/wbindkeys-test.log
--
-- and press the combos below.

local function log_cmd(name)
	return "echo \"$(date -Iseconds) fired: " .. name .. "\" >> testing/wbindkeys-test.log"
end

bind("ALT+A", log_cmd("ALT+A"))
bind("ALT+T", log_cmd("ALT+T"))

bind("Mouse1", log_cmd("Mouse1"))
bind("Mouse2", log_cmd("Mouse2"))
bind("Mouse3", log_cmd("Mouse3"))
bind("Mouse4", log_cmd("Mouse4"))
bind("Mouse5", log_cmd("Mouse5"))

bind("ScrollUp", log_cmd("ScrollUp"))
bind("ScrollDown", log_cmd("ScrollDown"))
bind("ScrollLeft", log_cmd("ScrollLeft"))
bind("ScrollRight", log_cmd("ScrollRight"))
