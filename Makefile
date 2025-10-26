# Rust project makefile
BIN='wbindkeys'

.Phony : build-debug build-release install clean run builddep

all: build-release

builddep:
	sudo apt install -y \
     cargo \
     libevdev-dev \
     libudev-dev \
     libinput-dev \


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

install: build-release
	cp target/release/$(BIN) ~/.local/bin/
