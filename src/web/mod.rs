pub mod embedded;
pub mod routes;
pub mod ws;

use crate::classifier::DeviceClassifier;
use crate::config::Config;
use crate::db::Database;
use crate::error::Result;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

pub async fn start_web_server(
    config: Arc<Config>,
    db: Arc<Database>,
    classifier: Arc<DeviceClassifier>,
    camera_flag: Arc<std::sync::atomic::AtomicBool>,
    tx: broadcast::Sender<String>,
) -> Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/status", get(routes::get_status))
        .route("/api/system/sources", get(routes::get_system_sources))
        .route("/api/system/config", post(routes::save_system_config))
        .route("/api/system/camera/status", get(routes::get_camera_status))
        .route("/api/system/camera/toggle", post(routes::toggle_camera))
        .route("/api/system/camera/test", post(routes::test_camera_device))
        .route("/api/system/storage/space", get(routes::get_storage_info))
        .route("/api/system/nas/config", post(routes::save_nas_config))
        .route("/api/backup/export", get(routes::export_backup))
        .route("/api/backup/restore", post(routes::restore_backup))
        .route("/api/devices", get(routes::get_devices))
        .route("/api/devices/:mac/history", get(routes::get_device_history))
        .route("/api/queries", get(routes::get_queries))
        .route("/api/queries/run", post(routes::run_query))
        .route("/api/oui/update", post(routes::update_oui))
        .route("/api/videos", get(routes::get_videos))
        .route("/api/videos/:id/timeline", get(routes::get_video_timeline))
        .route("/ws", get(ws::ws_handler))
        .nest_service("/videos", ServeDir::new(&config.video_dir))
        .fallback(embedded::static_handler)
        .layer(cors)
        .layer(Extension(config.clone()))
        .layer(Extension(db))
        .layer(Extension(classifier))
        .layer(Extension(camera_flag))
        .layer(Extension(tx));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.http_port));
    info!("Starting Web UI Dashboard on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
