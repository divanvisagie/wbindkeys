# Rust project makefile
BIN='wbindkeys'
VERSION=$(shell cargo pkgid | sed 's/.*[#@]//')

.DEFAULT_GOAL := help

.PHONY: help all builddep permissions build-debug build-release install clean check run test-run e2e publish-check publish docs

help: ## Show this help
	@echo "Usage: make <target>"
	@echo
	@awk 'BEGIN { FS = ":.*## " } /^[a-z-]+:.*## / { printf "  \033[1m%-14s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

all: build-release ## Build the release binary

builddep: ## Install Rust and build dependencies (Debian/Ubuntu)
	command -v rustc >/dev/null 2>&1 || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	sudo apt-get update
	sudo apt-get install -y pkg-config libevdev-dev libudev-dev libinput-dev

# Installs the udev rule that lets wbindkeys read input devices as your user
# and re-triggers udev so existing devices get access. Safe to re-run, e.g.
# when --debug shows "Permission denied" for input devices.
permissions: build-debug ## Grant your user access to input devices (sudo)
	sudo ./target/debug/$(BIN) permissions --set

build-debug: src/main.rs ## Build the debug binary
	cargo build

build-release: src/main.rs ## Build the release binary
	cargo build --release

clean: ## Remove build artifacts
	cargo clean

check: ## Type-check without building
	cargo check

run: build-debug ## Build and run with your config
	./target/debug/$(BIN)

# Runs wbindkeys against testing/config/wbindkeys/init.lua instead of the
# real user config, so PR branches can be verified locally: bindings in
# that file log to testing/wbindkeys-test.log (gitignored) instead of
# launching real applications. tail -f testing/wbindkeys-test.log while
# this runs and press the bound combos.
#
# Uses sudo regardless of `wbindkeys permissions --set`'s udev/seat ACL setup, so
# this keeps working for local testing even before that has been run.
test-run: build-debug ## Run against the test config (sudo)
	XDG_CONFIG_HOME=$(CURDIR)/testing/config sudo --preserve-env=XDG_CONFIG_HOME ./target/debug/$(BIN)

# Runs wbindkeys in an LXD virtual machine (created on first run) and
# presses key combos on a virtual keyboard there, checking which bindings
# fire. See testing/vm/e2e.sh for options.
e2e: build-release ## Run the end-to-end test in an LXD VM
	./testing/vm/e2e.sh target/release/$(BIN)

install: build-release ## Install to ~/.local/bin and start the user service
	./scripts/install.sh

# Regenerates the documentation website in docs/ (served by GitHub Pages)
# from README.md, man/wbindkeys.1 and LICENSE, after linting the man page
# with mandoc. Needs pandoc and mandoc.
docs: README.md man/wbindkeys.1 LICENSE templates/document.html templates/nav.html ## Regenerate the docs site in docs/
	./scripts/generate_docs.sh

# Builds the crate exactly as it would be uploaded to crates.io, without
# uploading anything.
publish-check: ## Test and dry-run the crates.io publish
	cargo test
	cargo publish --dry-run --allow-dirty

# Publishes the current version to crates.io and tags it as v$(VERSION).
# Only runs from a clean main branch; crates.io versions can't be replaced,
# so bump the version in Cargo.toml first. Push the tag afterwards with
# `git push origin v$(VERSION)`.
publish: ## Publish to crates.io and tag the version (from main)
	@test "$$(git rev-parse --abbrev-ref HEAD)" = main || { echo "error: publish from the main branch"; exit 1; }
	@test -z "$$(git status --porcelain)" || { echo "error: working tree has uncommitted changes"; exit 1; }
	@! git rev-parse -q --verify "refs/tags/v$(VERSION)" >/dev/null || { echo "error: tag v$(VERSION) already exists, bump the version in Cargo.toml"; exit 1; }
	cargo test
	cargo publish
	git tag -a "v$(VERSION)" -m "wbindkeys $(VERSION)"
