# Raspberry Pi BLE OPSEC Sniffer, Motion Correlator & Ecosystem

[![CI Build Pipeline](https://github.com/tekk/rpi-portable-ble-opsec-sniffer/actions/workflows/ci.yml/badge.svg)](https://github.com/tekk/rpi-portable-ble-opsec-sniffer/actions/workflows/ci.yml)
[![Automated Release Pipeline](https://github.com/tekk/rpi-portable-ble-opsec-sniffer/actions/workflows/release.yml/badge.svg)](https://github.com/tekk/rpi-portable-ble-opsec-sniffer/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/tekk/rpi-portable-ble-opsec-sniffer)](https://github.com/tekk/rpi-portable-ble-opsec-sniffer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-cyan.svg)](LICENSE)

An advanced, 24/7 Bluetooth Low Energy (BLE) promiscuous sniffer, IEEE OUI fingerprinting engine, motion video recorder, and ecosystem for Raspberry Pi (RPi 3/4/5), Home Assistant, and ESP32 hardware.

---

## Workspace Projects

### 1. 🦀 Main Rust Application (`/src`, `./build.fish`)
A high-performance Rust 2021 application for Raspberry Pi running 24/7:
- **Raw Linux HCI Sockets**: Direct `AF_BLUETOOTH` promiscuous scanning across multiple HCI adapters (`hci0`, `hci1`, Sena UD-100 CSR dongle, RPi 5 internal BT, BT 5.4/6.0 adapters).
- **IEEE OUI & 15+ Device Classifier**: Local IEEE vendor database with auto-download and multi-factor heuristics (iPhone, Android, AirTag, ESP32, nRF, Smartwatch, TV, Windows PC, Mac, Beacons, Smart Home, Headphones, HID, Medical, Automotive).
- **-10s / +10s Interleaved Motion Video Correlator**: V4L2 camera capture storing 10s pre-roll buffer, motion detection, post-roll recording, and automatic correlation of BLE devices active during motion videos.
- **SQLite WAL Database & 20+ Predefined Queries**: Indexed SQLite database with 20 sample queries and interactive Query Studio.
- **Embedded Web Dashboard**: Modern dark mode UI with live BLE proximity radar sweep, real-time WebSockets stream, and motion video player.

### 2. 🏠 Home Assistant Custom Component for HACS (`/custom_components/rpi_ble_opsec/`, `hacs.json`)
A modern HACS-compatible Home Assistant integration written in Python:
- **Entities**: Sensors for active BLE devices, AirTag/tracker detection count, recorded motion events, and live packet stream.
- **Camera Entity**: `camera.rpi_ble_motion_camera_feed` serving latest motion video snapshots.
- **Buttons & Services**: `button.update_ieee_oui_database` to trigger remote OUI database updates.
- **Real-Time WebSockets Listener**: Fires `rpi_ble_opsec_packet_received` events on Home Assistant bus for real-time automation triggers.

### 3. ⚡ ESPHome Microcontroller Project (`/esphome/`)
ESPHome firmware configuration (`.yaml` + custom C++ headers `oui_classifier.h` & `motion_recorder.h`):
- Runs on ESP32 / ESP32-CAM / ESP32-S3 boards.
- Promiscuous 24/7 BLE scanning (`esp32_ble_tracker`) with hardware OUI lookup.
- ESP32-CAM motion detection frame differencing and Home Assistant native API publishing.

---

## Quick Start Guide

### Bash & Fish Shell Build Helpers
```bash
# Make build scripts executable
chmod +x build.sh build.fish

# Check code for errors
./build.sh check   # or ./build.fish check

# Build release binary
./build.sh release # or ./build.fish release

# Run sniffer on hci0 (Port 8080)
./build.sh run hci0
```

### Cross-Compilation for Raspberry Pi
```bash
# Cross-compile for Raspberry Pi 4 / 5 64-bit (aarch64)
./build.sh rpi64   # or ./build.fish rpi64

# Cross-compile for Raspberry Pi 3 / 4 32-bit (armv7)
./build.sh rpi32   # or ./build.fish rpi32
```

---

## 🚀 Running as a Systemd Service (24/7 Background Daemon)

To install and run the app automatically as a 24/7 system service:

```bash
# Run installer as root (Bash or Fish shell)
sudo ./install-service.sh   # or sudo ./install-service.fish

# Enable and start immediately
sudo systemctl enable --now rpi-ble-opsec.service

# Check service status
sudo systemctl status rpi-ble-opsec.service

# Stream live systemd logs
sudo journalctl -u rpi-ble-opsec.service -f
```
