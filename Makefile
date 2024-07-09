# Rust project makefile
BIN='wbindkeys'

.Phony : build clean run

build: src/main.rs
	cargo build

clean: 
	cargo clean

run: build
	sudo ./target/debug/$(BIN)

