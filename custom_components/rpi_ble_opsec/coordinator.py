"""DataUpdateCoordinator for Raspberry Pi BLE OPSEC Sniffer."""
from __future__ import annotations

import async_timeout
import asyncio
import logging
from datetime import timedelta
import aiohttp

from homeassistant.core import HomeAssistant
from homeassistant.helpers.update_coordinator import DataUpdateCoordinator, UpdateFailed
from homeassistant.const import CONF_HOST, CONF_PORT

_LOGGER = logging.getLogger(__name__)


class BleOpsecDataUpdateCoordinator(DataUpdateCoordinator):
    """Class to manage fetching BLE OPSEC Sniffer data from REST & WebSockets API."""

    def __init__(self, hass: HomeAssistant, host: str, port: int) -> None:
        """Initialize coordinator."""
        self.host = host
        self.port = port
        self.base_url = f"http://{host}:{port}"
        self.ws_url = f"ws://{host}:{port}/ws"

        self.latest_devices = []
        self.latest_videos = []
        self.status_data = {}
        self.live_packet_count = 0
        self.trackers_count = 0

        super().__init__(
            hass,
            _LOGGER,
            name="rpi_ble_opsec_coordinator",
            update_interval=timedelta(seconds=10),
        )

        # Start WebSocket task
        hass.async_create_background_task(
            self._listen_websocket(), "rpi_ble_opsec_ws_listener"
        )

    async def _async_update_data(self) -> dict:
        """Fetch status, devices, and motion videos data from REST API."""
        try:
            async with async_timeout.timeout(10):
                async with aiohttp.ClientSession() as session:
                    # Status API
                    async with session.get(f"{self.base_url}/api/status") as resp:
                        if resp.status == 200:
                            self.status_data = await resp.json()

                    # Devices API
                    async with session.get(f"{self.base_url}/api/devices") as resp:
                        if resp.status == 200:
                            data = await resp.json()
                            self.latest_devices = data.get("devices", [])

                    # Videos API
                    async with session.get(f"{self.base_url}/api/videos") as resp:
                        if resp.status == 200:
                            data = await resp.json()
                            self.latest_videos = data.get("videos", [])

            # Compute stats
            trackers = [
                d for d in self.latest_devices 
                if "Tracker" in d.get("inferred_type", "") or "AirTag" in d.get("inferred_type", "")
            ]
            self.trackers_count = len(trackers)

            return {
                "status": self.status_data,
                "devices": self.latest_devices,
                "videos": self.latest_videos,
                "total_devices_count": len(self.latest_devices),
                "trackers_count": self.trackers_count,
                "motion_videos_count": len(self.latest_videos),
                "live_packet_count": self.live_packet_count,
            }
        except Exception as err:
            raise UpdateFailed(f"Error communicating with BLE Sniffer API: {err}") from err

    async def _listen_websocket(self) -> None:
        """Listen to WebSockets live BLE packet stream from Rust app."""
        while True:
            try:
                async with aiohttp.ClientSession() as session:
                    async with session.ws_connect(self.ws_url) as ws:
                        _LOGGER.info("Connected to RPi BLE Sniffer WebSocket at %s", self.ws_url)
                        async for msg in ws:
                            if msg.type == aiohttp.WSMsgType.TEXT:
                                self.live_packet_count += 1
                                # Fire Home Assistant bus event for real-time automation triggers
                                self.hass.bus.async_fire("rpi_ble_opsec_packet_received", {
                                    "raw": msg.data
                                })
            except Exception as err:
                _LOGGER.warning("WebSocket disconnected (%s), reconnecting in 5s...", err)
                await asyncio.sleep(5)

    async fn_trigger_oui_update(self) -> bool:
        """Trigger remote IEEE OUI database update."""
        async with aiohttp.ClientSession() as session:
            async with session.post(f"{self.base_url}/api/oui/update") as resp:
                return resp.status == 200
