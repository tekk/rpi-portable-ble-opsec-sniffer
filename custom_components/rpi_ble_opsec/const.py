"""Constants for the Raspberry Pi BLE OPSEC Sniffer integration."""

DOMAIN = "rpi_ble_opsec"
DEFAULT_NAME = "RPi BLE OPSEC"
DEFAULT_PORT = 8080
DEFAULT_MQTT_PORT = 1883
DEFAULT_MQTT_TOPIC = "homeassistant/rpi_ble_opsec"

CONF_CONNECTION_TYPE = "connection_type"
CONF_HOST = "host"
CONF_PORT = "port"

CONF_MQTT_HOST = "mqtt_host"
CONF_MQTT_PORT = "mqtt_port"
CONF_MQTT_TOPIC = "mqtt_topic"
CONF_MQTT_USER = "mqtt_user"
CONF_MQTT_PASS = "mqtt_pass"

CONN_TYPE_REST = "REST / WebSockets API"
CONN_TYPE_MQTT = "MQTT Broker Subscription"

ATTR_MAC_ADDRESS = "mac_address"
ATTR_DEVICE_TYPE = "inferred_type"
ATTR_VENDOR_NAME = "vendor_name"
ATTR_MAX_RSSI = "max_rssi"
ATTR_TOTAL_PACKETS = "total_packets"
