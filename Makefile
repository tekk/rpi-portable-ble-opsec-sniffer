# Makefile for Raspberry Pi BLE OPSEC Sniffer & Motion Correlator

.PHONY: all check build release cross-aarch64 cross-armv7 run clean help

all: check build

check:
	cargo check

build:
	cargo build

release:
	cargo build --release

# Cross-compilation for RPi 4/5 64-bit OS
cross-aarch64:
	@echo "Building for Raspberry Pi 64-bit (aarch64-unknown-linux-gnu)..."
	cargo build --release --target aarch64-unknown-linux-gnu

# Cross-compilation for RPi 3/4 32-bit OS
cross-armv7:
	@echo "Building for Raspberry Pi 32-bit (armv7-unknown-linux-gnueabihf)..."
	cargo build --release --target armv7-unknown-linux-gnueabihf

run:
	cargo run -- --hci hci0 --http-port 8080

clean:
	cargo clean

help:
	@echo "Available make targets:"
	@echo "  make build          - Build debug binary locally"
	@echo "  make release        - Build optimized release binary locally"
	@echo "  make cross-aarch64  - Cross-compile for RPi 64-bit (aarch64)"
	@echo "  make cross-armv7    - Cross-compile for RPi 32-bit (armv7)"
	@echo "  make run            - Run locally on hci0"
