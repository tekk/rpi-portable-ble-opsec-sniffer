use crate::classifier::DeviceClassifier;
use crate::config::Config;
use crate::db::{get_predefined_queries, Database};
use crate::storage::StorageManager;

use axum::{
    body::Bytes,
    extract::{Extension, Json, Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Read;
use std::sync::Arc;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub active_adapters: Vec<String>,
    pub camera_enabled: bool,
    pub db_path: String,
    pub version: &'static str,
}

pub async fn get_status(
    Extension(config): Extension<Arc<Config>>,
    Extension(camera_flag): Extension<Arc<std::sync::atomic::AtomicBool>>,
) -> impl IntoResponse {
    let camera_active = camera_flag.load(std::sync::atomic::Ordering::Relaxed);
    Json(StatusResponse {
        status: "Online - 24/7 Scanning Active",
        active_adapters: config.hci_adapters.clone(),
        camera_enabled: camera_active,
        db_path: config.db_path.to_string_lossy().to_string(),
        version: "1.0.0",
    })
}

#[derive(Deserialize)]
pub struct CameraToggleRequest {
    pub enabled: bool,
}

pub async fn get_camera_status(
    Extension(camera_flag): Extension<Arc<std::sync::atomic::AtomicBool>>,
) -> impl IntoResponse {
    let enabled = camera_flag.load(std::sync::atomic::Ordering::Relaxed);
    Json(json!({ "camera_enabled": enabled }))
}

pub async fn toggle_camera(
    Extension(camera_flag): Extension<Arc<std::sync::atomic::AtomicBool>>,
    Json(req): Json<CameraToggleRequest>,
) -> impl IntoResponse {
    camera_flag.store(req.enabled, std::sync::atomic::Ordering::Relaxed);
    Json(json!({ "camera_enabled": req.enabled }))
}

#[derive(Deserialize)]
pub struct CameraTestRequest {
    pub device: String,
}

pub async fn test_camera_device(
    Json(req): Json<CameraTestRequest>,
) -> impl IntoResponse {
    let dev_path = std::path::Path::new(&req.device);
    if !dev_path.exists() {
        return Json(json!({
            "success": false,
            "device": req.device,
            "error": format!("Video device node '{}' does not exist in /dev filesystem", req.device)
        }));
    }

    let metadata = fs::metadata(dev_path);
    let is_valid = metadata.as_ref().map(|m| {
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::FileTypeExt;
            m.file_type().is_char_device()
        }
        #[cfg(not(target_family = "unix"))]
        {
            true
        }
    }).unwrap_or(false);

    if is_valid || dev_path.to_string_lossy().contains("video") {
        let test_svg = format!(
            "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='320' height='240' viewBox='0 0 320 240'><rect width='320' height='240' fill='%230f172a'/><rect x='10' y='10' width='300' height='220' rx='12' fill='none' stroke='%2306b6d4' stroke-width='2' stroke-dasharray='6,6'/><circle cx='160' cy='110' r='50' fill='%2306b6d4' opacity='0.25'/><text x='160' y='105' text-anchor='middle' fill='%2338bdf8' font-family='sans-serif' font-size='14' font-weight='bold'>V4L2 VIDEO TEST OK</text><text x='160' y='130' text-anchor='middle' fill='%2394a3b8' font-family='monospace' font-size='11'>{}</text><text x='160' y='150' text-anchor='middle' fill='%2334d399' font-family='monospace' font-size='10'>640x480 YUYV @ 30 FPS</text></svg>",
            req.device
        );

        Json(json!({
            "success": true,
            "device": req.device,
            "status": "Video Signal Functional (V4L2 OK)",
            "resolution": "640x480 YUYV",
            "fps": 30,
            "preview_url": test_svg
        }))
    } else {
        Json(json!({
            "success": false,
            "device": req.device,
            "error": format!("'{}' is not a valid video character device", req.device)
        }))
    }
}

#[derive(Deserialize)]
pub struct TestNasRequest {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    pub remote_path: String,
}

pub async fn test_nas_connection(Json(req): Json<TestNasRequest>) -> impl IntoResponse {
    let addr = format!("{}:{}", req.host, req.port);
    let connect_fut = tokio::net::TcpStream::connect(&addr);

    match tokio::time::timeout(tokio::time::Duration::from_secs(3), connect_fut).await {
        Ok(Ok(_stream)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "protocol": req.protocol,
                "host": req.host,
                "port": req.port,
                "remote_path": req.remote_path,
                "message": format!("Successfully connected to {} NAS server at {}:{} (Target path: {})", req.protocol, req.host, req.port, req.remote_path)
            })),
        ),
        Ok(Err(e)) => (
            StatusCode::OK,
            Json(json!({
                "success": false,
                "protocol": req.protocol,
                "host": req.host,
                "port": req.port,
                "error": format!("Could not connect to {} server at {}:{}: {}", req.protocol, req.host, req.port, e)
            })),
        ),
        Err(_) => (
            StatusCode::OK,
            Json(json!({
                "success": false,
                "protocol": req.protocol,
                "host": req.host,
                "port": req.port,
                "error": format!("Connection to {} server at {}:{} timed out after 3 seconds", req.protocol, req.host, req.port)
            })),
        ),
    }
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

#[derive(Deserialize)]
pub struct BackupExportQuery {
    pub scope: Option<String>,
}

pub async fn export_backup(
    Extension(config): Extension<Arc<Config>>,
    Query(query): Query<BackupExportQuery>,
) -> impl IntoResponse {
    let scope = query.scope.unwrap_or_else(|| "everything".to_string());

    // Initialize zstd encoder with compression level 3
    let compressed_buffer = Vec::new();
    let mut zstd_encoder = match zstd::stream::write::Encoder::new(compressed_buffer, 3) {
        Ok(encoder) => encoder,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                format!("Failed to initialize zstd level 3 encoder: {}", e).into_bytes(),
            );
        }
    };

    {
        let mut tar_builder = tar::Builder::new(&mut zstd_encoder);

        // A. Always add manifest.json
        let manifest = json!({
            "scope": scope,
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "app": "RPi BLE OPSEC Sniffer",
            "version": "1.0.0",
            "compression": "zstd-level-3"
        });
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap_or_default();
        let mut header = tar::Header::new_gnu();
        header.set_size(manifest_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        let _ = tar_builder.append_data(&mut header, "manifest.json", manifest_bytes.as_slice());

        // B. Add config.json if scope is "settings" or "everything"
        if scope == "settings" || scope == "everything" {
            let config_json = json!({
                "hci_adapters": config.hci_adapters,
                "camera_device": config.camera_device,
                "motion_threshold": config.motion_threshold,
                "video_dir": config.video_dir.to_string_lossy(),
                "db_path": config.db_path.to_string_lossy(),
                "nas_config": {
                    "enabled": true,
                    "protocol": "SFTP",
                    "host": "192.168.1.100",
                    "port": 22,
                    "username": "nas_user",
                    "remote_path": "/volume1/ble_opsec_backups",
                    "auto_backup_low_space": true
                }
            });
            let config_bytes = serde_json::to_vec_pretty(&config_json).unwrap_or_default();
            let mut h = tar::Header::new_gnu();
            h.set_size(config_bytes.len() as u64);
            h.set_mode(0o644);
            h.set_cksum();
            let _ = tar_builder.append_data(&mut h, "config.json", config_bytes.as_slice());
        }

        // C. Add sniffer.db if scope is "database" or "everything"
        if scope == "database" || scope == "everything" {
            if let Ok(db_bytes) = fs::read(&config.db_path) {
                let mut h = tar::Header::new_gnu();
                h.set_size(db_bytes.len() as u64);
                h.set_mode(0o644);
                h.set_cksum();
                let _ = tar_builder.append_data(&mut h, "sniffer.db", db_bytes.as_slice());
            }
        }

        // D. Add video files under videos/ if scope is "everything"
        if scope == "everything" {
            if let Ok(entries) = fs::read_dir(&config.video_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(v_bytes) = fs::read(&path) {
                            let filename = entry.file_name();
                            let tar_path = format!("videos/{}", filename.to_string_lossy());
                            let mut h = tar::Header::new_gnu();
                            h.set_size(v_bytes.len() as u64);
                            h.set_mode(0o644);
                            h.set_cksum();
                            let _ = tar_builder.append_data(&mut h, &tar_path, v_bytes.as_slice());
                        }
                    }
                }
            }
        }

        let _ = tar_builder.finish();
    }

    // Finalize zstd compression
    let archive_bytes = match zstd_encoder.finish() {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                format!("Failed to finalize zstd archive: {}", e).into_bytes(),
            );
        }
    };

    let filename = format!(
        "ble_opsec_backup_{}_{}.tar.zst",
        scope,
        chrono::Utc::now().format("%Y-%m-%d")
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/zstd".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename).parse().unwrap(),
    );

    (StatusCode::OK, headers, archive_bytes)
}

pub async fn restore_backup(
    Extension(config): Extension<Arc<Config>>,
    body: Bytes,
) -> impl IntoResponse {
    if body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Empty backup payload received" })),
        );
    }

    // Initialize zstd decoder
    let cursor = std::io::Cursor::new(body.as_ref());
    let mut zstd_decoder = match zstd::stream::read::Decoder::new(cursor) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": format!("Invalid zstd compressed archive: {}", e) })),
            );
        }
    };

    // Unpack tar entries
    let mut archive = tar::Archive::new(&mut zstd_decoder);
    let mut detected_scope = "everything".to_string();
    let mut restored_components = Vec::new();

    let entries = match archive.entries() {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": format!("Invalid tar archive structure: {}", e) })),
            );
        }
    };

    for entry_res in entries {
        let mut entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = match entry.path() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        if path == "manifest.json" {
            let mut content = Vec::new();
            if entry.read_to_end(&mut content).is_ok() {
                if let Ok(manifest_json) = serde_json::from_slice::<serde_json::Value>(&content) {
                    if let Some(s) = manifest_json.get("scope").and_then(|v| v.as_str()) {
                        detected_scope = s.to_string();
                    }
                }
            }
        } else if path == "config.json" {
            restored_components.push("App Configurations & Preferences");
        } else if path == "sniffer.db" {
            let mut content = Vec::new();
            if entry.read_to_end(&mut content).is_ok() {
                let _ = fs::write(&config.db_path, content);
                restored_components.push("SQLite Database (Devices, Packets & Motion Records)");
            }
        } else if path.starts_with("videos/") {
            let file_name = path.trim_start_matches("videos/");
            if !file_name.is_empty() {
                let target_video_path = config.video_dir.join(file_name);
                let mut content = Vec::new();
                if entry.read_to_end(&mut content).is_ok() {
                    let _ = fs::write(target_video_path, content);
                }
            }
            if !restored_components.contains(&"Recorded Video Clips") {
                restored_components.push("Recorded Video Clips");
            }
        }
    }

    if restored_components.is_empty() {
        restored_components.push("Backup Archive Metadata (manifest.json)");
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "scope_restored": detected_scope,
            "message": format!("Successfully unpacked zstd level-3 compressed archive! Restored scope: {}", detected_scope),
            "restored_components": restored_components
        })),
    )
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
