# ESP32 BLE OPSEC Sniffer & Motion Recorder (ESPHome Firmware)

An ESPHome project designed for ESP32 and ESP32-CAM microcontrollers that replicates the core functionality of the main Raspberry Pi BLE OPSEC Sniffer app in hardware.

## Features
- **24/7 Promiscuous BLE Tracker**: Captures all nearby Bluetooth Low Energy advertisement packets.
- **On-Chip Device Classifier (`oui_classifier.h`)**: IEEE OUI lookup & device type inference (iPhone, Android, AirTag, ESP32, nRF, Smartwatch, TV).
- **ESP32-CAM Motion Capture (`motion_recorder.h`)**: Frame difference motion detection with motion score sensors and camera image stream.
- **Home Assistant Integration**: Native Home Assistant API integration, Web Server, and REST endpoints.

## Flashing Instructions
1. Install ESPHome CLI:
   ```bash
   pip install esphome
   ```
2. Edit `rpi_ble_opsec_esp32.yaml` with your Wi-Fi credentials (`ssid` & `password`).
3. Compile and flash your ESP32 / ESP32-CAM board:
   ```bash
   esphome run rpi_ble_opsec_esp32.yaml
   ```
