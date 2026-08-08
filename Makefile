# Rust project makefile
BIN='wbindkeys'

.Phony : builddep build-debug build-release install clean check run test-run

all: build-release

builddep:
	command -v rustc >/dev/null 2>&1 || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	sudo apt-get update
	sudo apt-get install -y pkg-config libevdev-dev libudev-dev libinput-dev

build-debug: src/main.rs
	cargo build

build-release: src/main.rs
	cargo build --release

clean:
	cargo clean

check:
	cargo check

run: build-debug
	./target/debug/$(BIN)

# Runs wbindkeys against testing/config/wbindkeys/init.lua instead of the
# real user config, so PR branches can be verified locally: bindings in
# that file log to testing/wbindkeys-test.log (gitignored) instead of
# launching real applications. tail -f testing/wbindkeys-test.log while
# this runs and press the bound combos.
#
# Uses sudo regardless of scripts/permissions.sh's udev/seat ACL setup, so
# this keeps working for local testing even before that script has been run.
test-run: build-debug
	XDG_CONFIG_HOME=$(CURDIR)/testing/config sudo --preserve-env=XDG_CONFIG_HOME ./target/debug/$(BIN)

install: build-release
	cp target/release/$(BIN) ~/.local/bin/
