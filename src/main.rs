mod ble;
mod camera;
mod classifier;
mod config;
mod db;
mod error;
mod mqtt;
mod storage;
mod web;

use crate::ble::HciScanner;
use crate::camera::CameraManager;
use crate::classifier::DeviceClassifier;
use crate::config::Config;
use crate::error::Result;

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Tracing Subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default tracing subscriber");

    info!("===========================================================");
    info!("Starting Raspberry Pi BLE OPSEC Sniffer & Motion Correlator");
    info!("===========================================================");

    // 2. Parse Configuration
    let config = Arc::new(Config::parse_args());
    info!("Configured HCI Adapters: {:?}", config.hci_adapters);
    info!("Configured Camera Device: {}", config.camera_device);
    info!("HTTP Server Port: {}", config.http_port);

    // 3. Initialize Database
    let database = Arc::new(db::open_database(&config.db_path)?);

    // 4. Initialize Device Fingerprinting & OUI Lookup Engine
    let classifier = Arc::new(DeviceClassifier::new());
    if config.auto_download_oui {
        let classifier_clone = classifier.clone();
        tokio::spawn(async move {
            let _ = classifier_clone.update_oui_db().await;
        });
    }

    // 5. Channels for BLE Packets (Internal & WebSocket / MQTT Broadcast)
    let (packet_tx, mut packet_rx) = mpsc::channel(1000);
    let (ws_tx, _) = broadcast::channel(100);

    // 6. Spawn MQTT Publisher Task if enabled
    let config_mqtt = config.clone();
    let rx_mqtt = ws_tx.subscribe();
    tokio::spawn(async move {
        let _ = mqtt::start_mqtt_publisher(config_mqtt, rx_mqtt).await;
    });

    // 7. Spawn BLE Scanner Task for each HCI adapter (hci0, hci1, UD-100, etc.)
    for adapter in config.hci_adapters.clone() {
        let scanner = HciScanner::new(&adapter, classifier.clone());
        let tx = packet_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = scanner.start_scan_loop(tx).await {
                tracing::error!("HCI Scanner error on {}: {}", adapter, e);
            }
        });
    }

    // 8. Spawn BLE Packet Processing Task (Database, WebSockets & MQTT)
    let db_clone = database.clone();
    let ws_tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        while let Some(packet) = packet_rx.recv().await {
            // Log packet to SQLite WAL Database
            let _ = db_clone.insert_packet(&packet);

            // Broadcast packet to WebSockets & MQTT connected subscribers
            if let Ok(json_str) = serde_json::to_string(&packet) {
                let _ = ws_tx_clone.send(json_str);
            }
        }
    });

    // 9. Spawn Camera Motion Detector & Video Correlator Task
    if config.camera_enabled {
        let camera_mgr = CameraManager::new(
            &config.camera_device,
            config.video_dir.clone(),
            config.motion_threshold,
            database.clone(),
        );
        tokio::spawn(async move {
            if let Err(e) = camera_mgr.start_capture_loop().await {
                tracing::error!("Camera Manager error: {}", e);
            }
        });
    }

    // 10. Start Axum Web Server & WebSockets Endpoint
    web::start_web_server(config, database, classifier, ws_tx).await?;

    Ok(())
}
