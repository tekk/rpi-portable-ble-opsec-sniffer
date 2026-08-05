#!/usr/bin/env bash
set -e

echo "=========================================================="
echo "Installing RPi BLE OPSEC Sniffer Systemd Service"
echo "=========================================================="

if [ "$EUID" -ne 0 ]; then
  echo "Error: Please run as root (e.g., sudo ./install-service.sh)"
  exit 1
fi

INSTALL_DIR="/opt/rpi-ble-opsec"
SERVICE_FILE="/etc/systemd/system/rpi-ble-opsec.service"

mkdir -p "$INSTALL_DIR/storage/videos"
mkdir -p "$INSTALL_DIR/static"

echo "Building release binary..."
cargo build --release

echo "Copying application files to $INSTALL_DIR..."
cp target/release/rpi-portable-ble-opsec-sniffer "$INSTALL_DIR/"
cp -r static/* "$INSTALL_DIR/static/" 2>/dev/null || true
cp rpi-ble-opsec.service "$SERVICE_FILE"

echo "Reloading systemd daemon..."
systemctl daemon-reload

echo "Enabling and starting rpi-ble-opsec service..."
systemctl enable --now rpi-ble-opsec.service

echo ""
echo "=========================================================="
echo "Installation complete!"
echo "Status: sudo systemctl status rpi-ble-opsec.service"
echo "Logs:   sudo journalctl -u rpi-ble-opsec.service -f"
echo "Web UI: http://<your-rpi-ip>:8080"
echo "=========================================================="
