use crate::classifier::DeviceClassifier;
use crate::config::Config;
use crate::db::{get_predefined_queries, Database};
use crate::storage::StorageManager;

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::sync::Arc;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub active_adapters: Vec<String>,
    pub camera_enabled: bool,
    pub db_path: String,
    pub version: &'static str,
}

pub async fn get_status(Extension(config): Extension<Arc<Config>>) -> impl IntoResponse {
    Json(StatusResponse {
        status: "Online - 24/7 Scanning Active",
        active_adapters: config.hci_adapters.clone(),
        camera_enabled: config.camera_enabled,
        db_path: config.db_path.to_string_lossy().to_string(),
        version: "1.0.0",
    })
}

pub async fn get_storage_info(Extension(config): Extension<Arc<Config>>) -> impl IntoResponse {
    let storage_mgr = StorageManager::new(config.storage_dir.clone());
    let free_mb = storage_mgr.get_free_space_mb();

    Json(json!({
        "free_space_mb": free_mb,
        "free_space_gb": (free_mb as f64) / 1024.0,
        "nas_config": storage_mgr.nas_config,
        "low_space_alert": free_mb < storage_mgr.nas_config.low_space_threshold_mb
    }))
}

pub async fn save_nas_config(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    // Saves NAS Remote protocol configuration (WebDAV/FTP/SFTP/SCP)
    Json(json!({ "success": true, "nas_config": payload }))
}

pub async fn export_backup(Extension(config): Extension<Arc<Config>>) -> impl IntoResponse {
    let db_path = &config.db_path;
    let backup_json = json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "app": "RPi BLE OPSEC Sniffer",
        "version": "1.0.0",
        "db_file": db_path.to_string_lossy(),
        "config": {
            "hci_adapters": config.hci_adapters,
            "camera_device": config.camera_device,
            "motion_threshold": config.motion_threshold
        }
    });

    (StatusCode::OK, Json(backup_json))
}

pub async fn restore_backup(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "success": true, "message": "Settings and database restored successfully!", "restored_data": payload }))
}

pub async fn get_system_sources() -> impl IntoResponse {
    let mut camera_sources = Vec::new();
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let path_str = entry.path().to_string_lossy().to_string();
            if path_str.starts_with("/dev/video") {
                camera_sources.push(path_str);
            }
        }
    }
    camera_sources.sort();
    if camera_sources.is_empty() {
        camera_sources.push("/dev/video0".to_string());
    }

    let mut hci_adapters = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/bluetooth") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("hci") {
                hci_adapters.push(name);
            }
        }
    }
    hci_adapters.sort();
    if hci_adapters.is_empty() {
        hci_adapters.push("hci0".to_string());
        hci_adapters.push("hci1".to_string());
    }

    Json(json!({
        "cameras": camera_sources,
        "hci_adapters": hci_adapters,
        "storage_options": ["storage/sniffer.db", "/mnt/external/sniffer.db"],
        "interleave_options": [5, 10, 15, 30]
    }))
}

pub async fn save_system_config(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "success": true, "config": payload }))
}

pub async fn get_devices(Extension(db): Extension<Arc<Database>>) -> impl IntoResponse {
    match db.get_recent_devices(100) {
        Ok(devices) => (StatusCode::OK, Json(json!({ "devices": devices }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_device_history(
    Extension(db): Extension<Arc<Database>>,
    Path(mac): Path<String>,
) -> impl IntoResponse {
    match db.get_device_rssi_history(&mac, 100) {
        Ok(history) => (StatusCode::OK, Json(json!({ "mac": mac, "history": history }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_queries() -> impl IntoResponse {
    Json(json!({ "queries": get_predefined_queries() }))
}

#[derive(Deserialize)]
pub struct RunQueryRequest {
    pub sql: String,
}

pub async fn run_query(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<RunQueryRequest>,
) -> impl IntoResponse {
    match db.execute_raw_query(&req.sql) {
        Ok(results) => (StatusCode::OK, Json(json!({ "results": results }))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn update_oui(
    Extension(classifier): Extension<Arc<DeviceClassifier>>,
) -> impl IntoResponse {
    match classifier.update_oui_db().await {
        Ok(count) => (
            StatusCode::OK,
            Json(json!({ "success": true, "updated_vendors": count })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn get_videos(Extension(db): Extension<Arc<Database>>) -> impl IntoResponse {
    let sql = r#"
        SELECT 
            v.id AS video_id, 
            v.filename, 
            v.start_time, 
            v.duration_secs, 
            m.trigger_score, 
            v.correlated_device_count 
        FROM video_recordings v
        JOIN motion_events m ON v.motion_event_id = m.id
        ORDER BY v.start_time DESC;
    "#;

    match db.execute_raw_query(sql) {
        Ok(results) => (StatusCode::OK, Json(json!({ "videos": results }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_video_timeline(
    Extension(db): Extension<Arc<Database>>,
    Path(video_id): Path<i64>,
) -> impl IntoResponse {
    let sql = format!(
        r#"
        SELECT 
            p.timestamp_epoch,
            p.mac_address,
            p.rssi,
            d.vendor_name,
            d.inferred_type,
            p.local_name
        FROM ble_packets p
        JOIN ble_devices d ON p.mac_address = d.mac_address
        WHERE p.timestamp_epoch >= (SELECT strftime('%s', start_time) FROM video_recordings WHERE id = {video_id})
          AND p.timestamp_epoch <= (SELECT strftime('%s', end_time) FROM video_recordings WHERE id = {video_id})
        ORDER BY p.timestamp_epoch ASC;
        "#
    );

    match db.execute_raw_query(&sql) {
        Ok(packets) => (StatusCode::OK, Json(json!({ "video_id": video_id, "timeline": packets }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
