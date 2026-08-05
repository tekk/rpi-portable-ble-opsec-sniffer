"""Sensor platform for Raspberry Pi BLE OPSEC Sniffer."""
from __future__ import annotations

from homeassistant.components.sensor import (
    SensorEntity,
    SensorStateClass,
)
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.helpers.update_coordinator import CoordinatorEntity

from .const import DOMAIN
from .coordinator import BleOpsecDataUpdateCoordinator


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    """Set up BLE OPSEC sensors from a config entry."""
    coordinator: BleOpsecDataUpdateCoordinator = hass.data[DOMAIN][entry.entry_id]

    sensors = [
        BleActiveDevicesSensor(coordinator, entry),
        BleTrackersSensor(coordinator, entry),
        MotionEventsSensor(coordinator, entry),
        LivePacketStreamSensor(coordinator, entry),
    ]

    async_add_entities(sensors)


class BleActiveDevicesSensor(CoordinatorEntity, SensorEntity):
    """Representation of the Active BLE Devices Count Sensor."""

    def __init__(
        self, coordinator: BleOpsecDataUpdateCoordinator, entry: ConfigEntry
    ) -> None:
        super().__init__(coordinator)
        self._attr_name = "RPi BLE Active Devices"
        self._attr_unique_id = f"{entry.entry_id}_active_devices"
        self._attr_icon = "mdi:bluetooth"
        self._attr_state_class = SensorStateClass.MEASUREMENT

    @property
    def native_value(self) -> int:
        """Return total unique logged BLE devices."""
        return self.coordinator.data.get("total_devices_count", 0)

    @property
    def extra_state_attributes(self) -> dict:
        """Return attributes containing recent devices list."""
        devices = self.coordinator.data.get("devices", [])
        return {
            "devices_summary": [
                {
                    "mac": d.get("mac_address"),
                    "type": d.get("inferred_type"),
                    "vendor": d.get("vendor_name"),
                    "rssi": d.get("max_rssi"),
                }
                for d in devices[:10]
            ]
        }


class BleTrackersSensor(CoordinatorEntity, SensorEntity):
    """Representation of AirTag & Tracker Devices Sensor."""

    def __init__(
        self, coordinator: BleOpsecDataUpdateCoordinator, entry: ConfigEntry
    ) -> None:
        super().__init__(coordinator)
        self._attr_name = "RPi BLE Trackers & AirTags Detected"
        self._attr_unique_id = f"{entry.entry_id}_trackers_detected"
        self._attr_icon = "mdi:tag-search"
        self._attr_state_class = SensorStateClass.MEASUREMENT

    @property
    def native_value(self) -> int:
        """Return total detected trackers count."""
        return self.coordinator.data.get("trackers_count", 0)


class MotionEventsSensor(CoordinatorEntity, SensorEntity):
    """Representation of Motion Events Count Sensor."""

    def __init__(
        self, coordinator: BleOpsecDataUpdateCoordinator, entry: ConfigEntry
    ) -> None:
        super().__init__(coordinator)
        self._attr_name = "RPi BLE Motion Video Events"
        self._attr_unique_id = f"{entry.entry_id}_motion_events"
        self._attr_icon = "mdi:motion-sensor-video"
        self._attr_state_class = SensorStateClass.TOTAL

    @property
    def native_value(self) -> int:
        """Return recorded motion video events count."""
        return self.coordinator.data.get("motion_videos_count", 0)


class LivePacketStreamSensor(CoordinatorEntity, SensorEntity):
    """Representation of Live Logged Packets Sensor."""

    def __init__(
        self, coordinator: BleOpsecDataUpdateCoordinator, entry: ConfigEntry
    ) -> None:
        super().__init__(coordinator)
        self._attr_name = "RPi BLE Packets Streamed"
        self._attr_unique_id = f"{entry.entry_id}_packets_streamed"
        self._attr_icon = "mdi:waveform"
        self._attr_state_class = SensorStateClass.TOTAL_INCREASING

    @property
    def native_value(self) -> int:
        """Return total live streamed packets."""
        return self.coordinator.data.get("live_packet_count", 0)
