#!/usr/bin/env bash
# build.sh - Helper script for Bash shell users on Raspberry Pi BLE Sniffer

set -e

show_help() {
    echo "Usage: ./build.sh [option]"
    echo "Options:"
    echo "  check         Check project code for errors"
    echo "  build         Build debug binary"
    echo "  release       Build release binary"
    echo "  rpi64         Cross-compile for Raspberry Pi 64-bit (aarch64)"
    echo "  rpi32         Cross-compile for Raspberry Pi 32-bit (armv7)"
    echo "  run [adapter] Run sniffer (default: hci0)"
}

case "$1" in
    check)
        cargo check
        ;;
    build)
        cargo build
        ;;
    release)
        cargo build --release
        ;;
    rpi64)
        echo "Building release binary for aarch64-unknown-linux-gnu..."
        cargo build --release --target aarch64-unknown-linux-gnu
        ;;
    rpi32)
        echo "Building release binary for armv7-unknown-linux-gnueabihf..."
        cargo build --release --target armv7-unknown-linux-gnueabihf
        ;;
    run)
        ADAPTER="${2:-hci0}"
        echo "Starting BLE Sniffer on adapter $ADAPTER..."
        cargo run -- --hci "$ADAPTER" --http-port 8080
        ;;
    *)
        show_help
        ;;
esac
