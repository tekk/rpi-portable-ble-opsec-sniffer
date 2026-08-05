"""Config flow for Raspberry Pi BLE OPSEC Sniffer integration with REST & MQTT support."""
from __future__ import annotations

import logging
import aiohttp
import voluptuous as vol

from homeassistant import config_entries
from homeassistant.const import CONF_HOST, CONF_PORT
from homeassistant.core import HomeAssistant
from homeassistant.data_entry_flow import FlowResult
import homeassistant.helpers.config_validation as cv

from .const import (
    DEFAULT_NAME,
    DEFAULT_PORT,
    DEFAULT_MQTT_PORT,
    DEFAULT_MQTT_TOPIC,
    DOMAIN,
    CONF_CONNECTION_TYPE,
    CONF_MQTT_HOST,
    CONF_MQTT_PORT,
    CONF_MQTT_TOPIC,
    CONF_MQTT_USER,
    CONF_MQTT_PASS,
    CONN_TYPE_REST,
    CONN_TYPE_MQTT,
)

_LOGGER = logging.getLogger(__name__)

STEP_USER_SCHEMA = vol.Schema(
    {
        vol.Required(CONF_CONNECTION_TYPE, default=CONN_TYPE_REST): vol.In(
            [CONN_TYPE_REST, CONN_TYPE_MQTT]
        ),
    }
)

STEP_REST_SCHEMA = vol.Schema(
    {
        vol.Required(CONF_HOST, default="localhost"): str,
        vol.Required(CONF_PORT, default=DEFAULT_PORT): cv.port,
    }
)

STEP_MQTT_SCHEMA = vol.Schema(
    {
        vol.Required(CONF_MQTT_HOST, default="localhost"): str,
        vol.Required(CONF_MQTT_PORT, default=DEFAULT_MQTT_PORT): cv.port,
        vol.Required(CONF_MQTT_TOPIC, default=DEFAULT_MQTT_TOPIC): str,
        vol.Optional(CONF_MQTT_USER): str,
        vol.Optional(CONF_MQTT_PASS): str,
    }
)


class ConfigFlow(config_entries.ConfigFlow, domain=DOMAIN):
    """Handle config flow for Raspberry Pi BLE OPSEC Sniffer."""

    VERSION = 2

    def __init__(self) -> None:
        """Initialize flow."""
        self._conn_type: str | None = None

    async def async_step_user(
        self, user_input: dict[str, any] | None = None
    ) -> FlowResult:
        """Handle connection method selection (REST vs MQTT)."""
        if user_input is not None:
            self._conn_type = user_input[CONF_CONNECTION_TYPE]
            if self._conn_type == CONN_TYPE_MQTT:
                return await self.async_step_mqtt()
            return await self.async_step_rest()

        return self.async_show_form(
            step_id="user", data_schema=STEP_USER_SCHEMA
        )

    async def async_step_rest(
        self, user_input: dict[str, any] | None = None
    ) -> FlowResult:
        """Handle REST / WebSockets API configuration."""
        errors: dict[str, str] = {}

        if user_input is not None:
            user_input[CONF_CONNECTION_TYPE] = CONN_TYPE_REST
            return self.async_create_entry(
                title=f"{DEFAULT_NAME} (REST - {user_input[CONF_HOST]})",
                data=user_input,
            )

        return self.async_show_form(
            step_id="rest", data_schema=STEP_REST_SCHEMA, errors=errors
        )

    async def async_step_mqtt(
        self, user_input: dict[str, any] | None = None
    ) -> FlowResult:
        """Handle MQTT Broker configuration."""
        if user_input is not None:
            user_input[CONF_CONNECTION_TYPE] = CONN_TYPE_MQTT
            return self.async_create_entry(
                title=f"{DEFAULT_NAME} (MQTT - {user_input[CONF_MQTT_HOST]})",
                data=user_input,
            )

        return self.async_show_form(
            step_id="mqtt", data_schema=STEP_MQTT_SCHEMA
        )
