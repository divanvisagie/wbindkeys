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

local log = "echo \"$(date -Iseconds) fired: "

bind("ALT+A", log .. "ALT+A\" >> testing/wbindkeys-test.log")
bind("ALT+T", log .. "ALT+T\" >> testing/wbindkeys-test.log")
