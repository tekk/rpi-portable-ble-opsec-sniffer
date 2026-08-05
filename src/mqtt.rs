use crate::ble::types::BlePacket;
use crate::config::Config;
use crate::error::Result;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{info, warn};

pub async fn start_mqtt_publisher(
    config: Arc<Config>,
    mut rx: broadcast::Receiver<String>,
) -> Result<()> {
    if !config.mqtt_enabled {
        return Ok(());
    }

    info!(
        "Initializing MQTT Client to broker {}:{} on topic '{}'...",
        config.mqtt_host, config.mqtt_port, config.mqtt_topic
    );

    let mut mqttoptions = MqttOptions::new("rpi-ble-opsec-sniffer", &config.mqtt_host, config.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(10));

    if let (Some(u), Some(p)) = (&config.mqtt_user, &config.mqtt_pass) {
        mqttoptions.set_credentials(u, p);
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 100);

    // Publish Home Assistant MQTT Discovery configuration payloads for 1-click HA integration!
    let discovery_topic = format!("{}/sensor/active_devices/config", config.mqtt_topic);
    let discovery_payload = json!({
        "name": "RPi BLE Active Devices",
        "state_topic": format!("{}/state", config.mqtt_topic),
        "value_template": "{{ value_json.active_devices_count }}",
        "unit_of_measurement": "devices",
        "icon": "mdi:bluetooth",
        "unique_id": "rpi_ble_opsec_mqtt_active_devices"
    });

    let _ = client
        .publish(
            &discovery_topic,
            QoS::AtLeastOnce,
            true,
            discovery_payload.to_string().into_bytes(),
        )
        .await;

    info!("Published Home Assistant MQTT Auto-Discovery configuration payload.");

    // Event Loop task to handle MQTT connection state
    let client_clone = client.clone();
    let topic_base = config.mqtt_topic.clone();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(packet) = serde_json::from_str::<BlePacket>(&msg) {
                // Publish live BLE packet payload to MQTT topic
                let packet_topic = format!("{}/packet/{}", topic_base, packet.mac_address.replace(':', ""));
                let _ = client_clone
                    .publish(&packet_topic, QoS::AtMostOnce, false, msg.into_bytes())
                    .await;
            }
        }
    });

    // Run MQTT eventloop worker
    tokio::spawn(async move {
        loop {
            if let Err(e) = eventloop.poll().await {
                warn!("MQTT connection eventloop error: {:?}. Retrying in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });

    Ok(())
}
