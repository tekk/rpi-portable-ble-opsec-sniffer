"""Camera entity platform for Raspberry Pi BLE OPSEC Sniffer."""
from __future__ import annotations

import aiohttp
from homeassistant.components.camera import Camera
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
    """Set up BLE OPSEC camera from a config entry."""
    coordinator: BleOpsecDataUpdateCoordinator = hass.data[DOMAIN][entry.entry_id]
    async_add_entities([BleMotionCamera(coordinator, entry)])


class BleMotionCamera(CoordinatorEntity, Camera):
    """Camera displaying the latest recorded motion clip thumbnail."""

    def __init__(
        self, coordinator: BleOpsecDataUpdateCoordinator, entry: ConfigEntry
    ) -> None:
        super().__init__(coordinator)
        Camera.__init__(self)
        self._attr_name = "RPi BLE Motion Camera Feed"
        self._attr_unique_id = f"{entry.entry_id}_motion_camera"

    async def async_camera_image(
        self, width: int | None = None, height: int | None = None
    ) -> bytes | None:
        """Fetch latest motion video frame snapshot."""
        videos = self.coordinator.data.get("videos", [])
        if not videos:
            return None

        latest = videos[0]
        filename = latest.get("filename", "")
        url = f"{self.coordinator.base_url}/videos/{filename}"

        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(url, timeout=5) as response:
                    if response.status == 200:
                        return await response.read()
        except Exception:
            return None

        return None
