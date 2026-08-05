use rusqlite::{Connection, Result};

pub fn init_database(conn: &Connection) -> Result<()> {
    // Enable WAL mode for high performance 24/7 concurrent reads/writes
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

        CREATE TABLE IF NOT EXISTS ble_devices (
            mac_address TEXT PRIMARY KEY,
            address_type TEXT NOT NULL,
            vendor_name TEXT NOT NULL,
            inferred_type TEXT NOT NULL,
            first_seen TEXT NOT NULL,
            last_seen TEXT NOT NULL,
            max_rssi INTEGER NOT NULL,
            last_rssi INTEGER NOT NULL,
            total_packets INTEGER NOT NULL DEFAULT 1,
            last_adv_name TEXT,
            last_raw_eir_hex TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ble_packets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            timestamp_epoch INTEGER NOT NULL,
            adapter_name TEXT NOT NULL,
            mac_address TEXT NOT NULL,
            rssi INTEGER NOT NULL,
            raw_eir_hex TEXT NOT NULL,
            parsed_json TEXT
        );

        CREATE TABLE IF NOT EXISTS motion_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            trigger_score REAL NOT NULL,
            thumbnail_path TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS video_recordings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            filepath TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            duration_secs INTEGER NOT NULL,
            motion_event_id INTEGER NOT NULL,
            correlated_device_count INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(motion_event_id) REFERENCES motion_events(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS video_ble_correlations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            video_id INTEGER NOT NULL,
            mac_address TEXT NOT NULL,
            avg_rssi REAL NOT NULL,
            max_rssi INTEGER NOT NULL,
            packet_count INTEGER NOT NULL,
            FOREIGN KEY(video_id) REFERENCES video_recordings(id) ON DELETE CASCADE,
            FOREIGN KEY(mac_address) REFERENCES ble_devices(mac_address)
        );

        CREATE TABLE IF NOT EXISTS custom_queries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            sql_query TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Performance Indexes
        CREATE INDEX IF NOT EXISTS idx_packets_mac ON ble_packets(mac_address);
        CREATE INDEX IF NOT EXISTS idx_packets_ts ON ble_packets(timestamp_epoch);
        CREATE INDEX IF NOT EXISTS idx_devices_type ON ble_devices(inferred_type);
        CREATE INDEX IF NOT EXISTS idx_devices_seen ON ble_devices(last_seen);
        CREATE INDEX IF NOT EXISTS idx_corr_video ON video_ble_correlations(video_id);
        "#,
    )?;

    Ok(())
}
