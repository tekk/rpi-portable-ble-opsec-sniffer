use crate::ble::types::{BleDeviceSummary, BlePacket};
use crate::error::Result;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn insert_packet(&self, packet: &BlePacket) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let raw_eir_hex = hex::encode(&packet.raw_eir);

        let parsed_json = serde_json::to_string(&json!({
            "local_name": packet.local_name,
            "manufacturer_id": packet.manufacturer_id,
            "appearance": packet.appearance,
            "service_uuids": packet.service_uuids,
            "tx_power": packet.tx_power
        }))
        .unwrap_or_default();

        conn.execute(
            "INSERT INTO ble_packets (timestamp, timestamp_epoch, adapter_name, mac_address, rssi, raw_eir_hex, parsed_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                packet.timestamp,
                packet.timestamp_epoch,
                packet.adapter_name,
                packet.mac_address,
                packet.rssi,
                raw_eir_hex,
                parsed_json
            ],
        )?;

        conn.execute(
            "INSERT INTO ble_devices (mac_address, address_type, vendor_name, inferred_type, first_seen, last_seen, max_rssi, last_rssi, total_packets, last_adv_name, last_raw_eir_hex)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?6, 1, ?7, ?8)
             ON CONFLICT(mac_address) DO UPDATE SET
                last_seen = excluded.last_seen,
                max_rssi = MAX(max_rssi, excluded.max_rssi),
                last_rssi = excluded.last_rssi,
                total_packets = total_packets + 1,
                last_adv_name = COALESCE(excluded.last_adv_name, last_adv_name),
                last_raw_eir_hex = excluded.last_raw_eir_hex,
                inferred_type = excluded.inferred_type,
                vendor_name = excluded.vendor_name",
            params![
                packet.mac_address,
                packet.address_type.as_str(),
                packet.vendor_name,
                packet.inferred_device_type,
                packet.timestamp,
                packet.rssi,
                packet.local_name,
                raw_eir_hex
            ],
        )?;

        Ok(())
    }

    pub fn get_recent_devices(&self, limit: usize) -> Result<Vec<BleDeviceSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mac_address, address_type, vendor_name, inferred_type, first_seen, last_seen, max_rssi, last_rssi, total_packets, last_adv_name, last_raw_eir_hex
             FROM ble_devices ORDER BY last_seen DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(BleDeviceSummary {
                mac_address: row.get(0)?,
                address_type: row.get(1)?,
                vendor_name: row.get(2)?,
                inferred_type: row.get(3)?,
                first_seen: row.get(4)?,
                last_seen: row.get(5)?,
                max_rssi: row.get(6)?,
                last_rssi: row.get(7)?,
                total_packets: row.get(8)?,
                last_adv_name: row.get(9)?,
                last_raw_eir_hex: row.get(10)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Fetches time-series RSSI, SNR, distance estimation, and movement trend for a device MAC.
    pub fn get_device_rssi_history(&self, mac: &str, limit: usize) -> Result<Vec<Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT timestamp, rssi, parsed_json FROM ble_packets WHERE mac_address = ?1 ORDER BY id DESC LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![mac, limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i8>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        let mut raw_history = Vec::new();
        for r in rows {
            let (ts, rssi, parsed_str) = r?;
            let tx_power: f64 = if let Some(s) = parsed_str {
                if let Ok(v) = serde_json::from_str::<Value>(&s) {
                    v.get("tx_power").and_then(|p| p.as_f64()).unwrap_or(-59.0)
                } else {
                    -59.0
                }
            } else {
                -59.0
            };

            // Log-distance path loss distance estimation model: d = 10 ^ ((TxPower - RSSI) / (10 * n))
            // n = 2.5 (Indoor / Obstacle Environment)
            let path_loss_n = 2.5;
            let exponent = (tx_power - (rssi as f64)) / (10.0 * path_loss_n);
            let estimated_distance_meters = (10.0f64.powf(exponent) * 100.0).round() / 100.0;

            // Estimated SNR (Noise Floor ~ -95 dBm)
            let estimated_snr = (rssi as f64) - (-95.0);

            raw_history.push((ts, rssi, estimated_snr, estimated_distance_meters));
        }

        // Reverse to chronological order (oldest to newest)
        raw_history.reverse();

        // Calculate movement trend (Approaching vs Receding vs Stationary)
        let mut points = Vec::new();
        for i in 0..raw_history.len() {
            let (ref ts, rssi, snr, dist) = raw_history[i];
            let trend = if i > 0 {
                let prev_dist = raw_history[i - 1].3;
                let diff = dist - prev_dist;
                if diff < -0.3 {
                    "Approaching 🚶➡️📡"
                } else if diff > 0.3 {
                    "Receding 📡➡️🚶"
                } else {
                    "Stationary 🛑"
                }
            } else {
                "Stationary 🛑"
            };

            points.push(json!({
                "timestamp": ts,
                "rssi": rssi,
                "snr": snr,
                "estimated_distance_m": dist,
                "trend": trend
            }));
        }

        Ok(points)
    }

    pub fn record_motion_event_and_correlate(
        &self,
        start_time: &str,
        end_time: &str,
        score: f64,
        filename: &str,
        filepath: &str,
        thumbnail: &str,
        duration_secs: u64,
    ) -> Result<(i64, i64, usize)> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO motion_events (start_time, end_time, trigger_score, thumbnail_path) VALUES (?1, ?2, ?3, ?4)",
            params![start_time, end_time, score, thumbnail],
        )?;
        let motion_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO video_recordings (filename, filepath, start_time, end_time, duration_secs, motion_event_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![filename, filepath, start_time, end_time, duration_secs, motion_id],
        )?;
        let video_id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            "SELECT mac_address, AVG(rssi), MAX(rssi), COUNT(*) 
             FROM ble_packets 
             WHERE timestamp >= ?1 AND timestamp <= ?2 
             GROUP BY mac_address",
        )?;

        let rows = stmt.query_map(params![start_time, end_time], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i8>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        let mut count = 0;
        for r in rows {
            let (mac, avg_rssi, max_rssi, p_count) = r?;
            conn.execute(
                "INSERT INTO video_ble_correlations (video_id, mac_address, avg_rssi, max_rssi, packet_count)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![video_id, mac, avg_rssi, max_rssi, p_count],
            )?;
            count += 1;
        }

        conn.execute(
            "UPDATE video_recordings SET correlated_device_count = ?1 WHERE id = ?2",
            params![count, video_id],
        )?;

        Ok((motion_id, video_id, count))
    }

    pub fn execute_raw_query(&self, sql: &str) -> Result<Value> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let col_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();

        let mut rows = stmt.query([])?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let mut obj = serde_json::Map::new();
            for (idx, name) in col_names.iter().enumerate() {
                let val_ref = row.get_ref(idx)?;
                let json_val = match val_ref {
                    rusqlite::types::ValueRef::Null => Value::Null,
                    rusqlite::types::ValueRef::Integer(i) => json!(i),
                    rusqlite::types::ValueRef::Real(r) => json!(r),
                    rusqlite::types::ValueRef::Text(t) => {
                        let s = std::str::from_utf8(t).unwrap_or("");
                        json!(s)
                    }
                    rusqlite::types::ValueRef::Blob(b) => json!(hex::encode(b)),
                };
                obj.insert(name.clone(), json_val);
            }
            results.push(Value::Object(obj));
        }

        Ok(Value::Array(results))
    }
}
