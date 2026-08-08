# Rust project makefile
BIN='wbindkeys'

.Phony : setup build clean run test-run

setup:
	command -v rustc >/dev/null 2>&1 || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	sudo apt-get update
	sudo apt-get install -y pkg-config libevdev-dev libudev-dev libinput-dev

build: src/main.rs
	cargo build

clean: 
	cargo clean

check: 
	cargo check

run: build
	sudo ./target/debug/$(BIN)

# Runs wbindkeys against testing/config/wbindkeys/init.lua instead of the
# real user config, so PR branches can be verified locally: bindings in
# that file log to testing/wbindkeys-test.log (gitignored) instead of
# launching real applications. tail -f testing/wbindkeys-test.log while
# this runs and press the bound combos.
test-run: build
	XDG_CONFIG_HOME=$(CURDIR)/testing/config sudo --preserve-env=XDG_CONFIG_HOME ./target/debug/$(BIN)

