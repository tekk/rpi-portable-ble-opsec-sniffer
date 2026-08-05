#!/usr/bin/env fish
# build.fish - Helper script for Fish Shell users on Raspberry Pi BLE Sniffer

function show_help
    echo "Usage: ./build.fish [option]"
    echo "Options:"
    echo "  check         Check project code for errors"
    echo "  build         Build debug binary"
    echo "  release       Build release binary"
    echo "  rpi64         Cross-compile for Raspberry Pi 64-bit (aarch64)"
    echo "  rpi32         Cross-compile for Raspberry Pi 32-bit (armv7)"
    echo "  run [adapter] Run sniffer (default: hci0)"
end

switch "$argv[1]"
    case check
        cargo check
    case build
        cargo build
    case release
        cargo build --release
    case rpi64
        echo "Building release binary for aarch64-unknown-linux-gnu..."
        cargo build --release --target aarch64-unknown-linux-gnu
    case rpi32
        echo "Building release binary for armv7-unknown-linux-gnueabihf..."
        cargo build --release --target armv7-unknown-linux-gnueabihf
    case run
        set -l adapter "hci0"
        if test (count $argv) -ge 2
            set adapter $argv[2]
        end
        echo "Starting BLE Sniffer on adapter $adapter..."
        cargo run -- --hci $adapter --http-port 8080
    case '*'
        show_help
end
