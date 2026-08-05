"""Button platform for Raspberry Pi BLE OPSEC Sniffer."""
from __future__ import annotations

from homeassistant.components.button import ButtonEntity
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
    """Set up BLE OPSEC buttons from a config entry."""
    coordinator: BleOpsecDataUpdateCoordinator = hass.data[DOMAIN][entry.entry_id]
    async_add_entities([OuiUpdateButton(coordinator, entry)])


class OuiUpdateButton(CoordinatorEntity, ButtonEntity):
    """Representation of Update OUI DB Button."""

    def __init__(
        self, coordinator: BleOpsecDataUpdateCoordinator, entry: ConfigEntry
    ) -> None:
        super().__init__(coordinator)
        self._attr_name = "Update IEEE OUI Database"
        self._attr_unique_id = f"{entry.entry_id}_update_oui"
        self._attr_icon = "mdi:database-refresh"

    async def async_press(self) -> None:
        """Trigger remote OUI database update."""
        await self.coordinator.fn_trigger_oui_update()
        await self.coordinator.async_request_refresh()
