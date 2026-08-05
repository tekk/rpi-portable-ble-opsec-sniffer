use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedQuery {
    pub id: usize,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub sql: &'static str,
}

pub fn get_predefined_queries() -> Vec<PredefinedQuery> {
    vec![
        PredefinedQuery {
            id: 1,
            title: "1. BLE Devices Correlated with Motion Videos",
            category: "OPSEC & Motion Correlation",
            description: "Lists all BLE devices present during recorded motion events with RSSI stats and packet count",
            sql: r#"
SELECT 
    v.filename AS video_file,
    m.start_time AS motion_start,
    c.mac_address,
    d.vendor_name,
    d.inferred_type,
    c.avg_rssi,
    c.max_rssi,
    c.packet_count
FROM video_ble_correlations c
JOIN video_recordings v ON c.video_id = v.id
JOIN motion_events m ON v.motion_event_id = m.id
JOIN ble_devices d ON c.mac_address = d.mac_address
ORDER BY m.start_time DESC, c.max_rssi DESC;
"#,
        },
        PredefinedQuery {
            id: 2,
            title: "2. Top 10 Most Active Devices (24h)",
            category: "Traffic Analysis",
            description: "Top 10 BLE devices by total logged advertisement packet count",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    total_packets, 
    max_rssi, 
    last_seen 
FROM ble_devices 
ORDER BY total_packets DESC 
LIMIT 10;
"#,
        },
        PredefinedQuery {
            id: 3,
            title: "3. Nearby AirTag & Tracker Detections",
            category: "Threat & Tracker Detection",
            description: "Detects Apple AirTags, FindMy items, Tile trackers, and Bluetooth beacons nearby",
            sql: r#"
SELECT 
    mac_address, 
    address_type, 
    vendor_name, 
    inferred_type, 
    max_rssi, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%Tracker%' OR inferred_type LIKE '%AirTag%' OR inferred_type LIKE '%Beacon%'
ORDER BY last_seen DESC;
"#,
        },
        PredefinedQuery {
            id: 4,
            title: "4. Target Apple iOS Devices (iPhones / iPads)",
            category: "Target Identification",
            description: "List of detected Apple iPhones, iPads, and iOS ecosystem devices",
            sql: r#"
SELECT 
    mac_address, 
    address_type, 
    vendor_name, 
    last_adv_name, 
    max_rssi, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%iPhone%' OR vendor_name LIKE '%Apple%'
ORDER BY max_rssi DESC;
"#,
        },
        PredefinedQuery {
            id: 5,
            title: "5. Android & Google FastPair Devices",
            category: "Target Identification",
            description: "List of Android phones, Google Pixel, Samsung Galaxy, and FastPair devices",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    max_rssi, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%Android%' OR vendor_name LIKE '%Samsung%' OR vendor_name LIKE '%Google%'
ORDER BY last_seen DESC;
"#,
        },
        PredefinedQuery {
            id: 6,
            title: "6. ESP32 & IoT Embedded Hardware Log",
            category: "Hardware & IoT Sniffing",
            description: "Identify Espressif ESP32, ESP8266, and custom embedded BLE nodes",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    total_packets, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%ESP32%' OR vendor_name LIKE '%Espressif%'
ORDER BY total_packets DESC;
"#,
        },
        PredefinedQuery {
            id: 7,
            title: "7. Nordic Semiconductor nRF Devices",
            category: "Hardware & IoT Sniffing",
            description: "Locate Nordic nRF52/nRF53 hardware, development kits, and NUS serial nodes",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    max_rssi, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%nRF%' OR vendor_name LIKE '%Nordic%'
ORDER BY max_rssi DESC;
"#,
        },
        PredefinedQuery {
            id: 8,
            title: "8. Signal Strength (RSSI) Leaderboard",
            category: "Proximity & Range",
            description: "Devices closest to the sniffer based on highest RSSI signal strength",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    max_rssi, 
    last_rssi, 
    last_seen 
FROM ble_devices 
ORDER BY max_rssi DESC 
LIMIT 15;
"#,
        },
        PredefinedQuery {
            id: 9,
            title: "9. Unidentified MACs with Strong Signal (High RSSI)",
            category: "Anomaly Detection",
            description: "Unclassified devices with RSSI > -60 dBm that may be suspicious or unknown",
            sql: r#"
SELECT 
    mac_address, 
    address_type, 
    max_rssi, 
    last_seen 
FROM ble_devices 
WHERE vendor_name = 'Unknown Vendor' AND max_rssi > -60
ORDER BY max_rssi DESC;
"#,
        },
        PredefinedQuery {
            id: 10,
            title: "10. Smartwatches & Wearables Activity",
            category: "Device Inventory",
            description: "Log of Apple Watch, WearOS, Garmin, and Fitbit smartwatches",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    max_rssi, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%Smartwatch%' OR inferred_type LIKE '%Wearable%'
ORDER BY last_seen DESC;
"#,
        },
        PredefinedQuery {
            id: 11,
            title: "11. Recently First-Seen BLE Devices (New MACs)",
            category: "OPSEC & Perimeter",
            description: "Devices detected for the first time in the last 1 hour",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    first_seen, 
    max_rssi 
FROM ble_devices 
ORDER BY first_seen DESC 
LIMIT 15;
"#,
        },
        PredefinedQuery {
            id: 12,
            title: "12. Resolvable Private Address (RPA) MAC Frequency",
            category: "Privacy & Anti-Tracking",
            description: "Count of devices using randomized privacy MAC addresses vs static Public MACs",
            sql: r#"
SELECT 
    address_type, 
    COUNT(*) AS device_count, 
    SUM(total_packets) AS total_packets 
FROM ble_devices 
GROUP BY address_type;
"#,
        },
        PredefinedQuery {
            id: 13,
            title: "13. Smart TV & Media Streaming Devices",
            category: "Device Inventory",
            description: "List of LG, Samsung, Apple TV, Roku, and Smart TVs nearby",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%TV%' OR vendor_name LIKE '%LG%' OR vendor_name LIKE '%Roku%'
ORDER BY last_seen DESC;
"#,
        },
        PredefinedQuery {
            id: 14,
            title: "14. HID Keyboards, Mice & Input Devices",
            category: "Peripheral Sniffing",
            description: "Detect wireless Bluetooth keyboards, mice, and gamepads",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    inferred_type, 
    last_adv_name, 
    max_rssi 
FROM ble_devices 
WHERE inferred_type LIKE '%HID%'
ORDER BY max_rssi DESC;
"#,
        },
        PredefinedQuery {
            id: 15,
            title: "15. Motion Event Video Summary",
            category: "Video Analytics",
            description: "Summary of recorded motion videos and duration",
            sql: r#"
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
"#,
        },
        PredefinedQuery {
            id: 16,
            title: "16. Vendor Breakdown Percentage",
            category: "Vendor Distribution",
            description: "Breakdown of detected BLE devices by hardware manufacturer",
            sql: r#"
SELECT 
    vendor_name, 
    COUNT(*) AS count, 
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM ble_devices), 2) AS percentage 
FROM ble_devices 
GROUP BY vendor_name 
ORDER BY count DESC;
"#,
        },
        PredefinedQuery {
            id: 17,
            title: "17. Raw Packet Logs with EIR Payload (Latest 20)",
            category: "Packet Inspection",
            description: "Inspect recent raw BLE advertisement packets with RSSI and payload hex",
            sql: r#"
SELECT 
    id, 
    timestamp, 
    adapter_name, 
    mac_address, 
    rssi, 
    raw_eir_hex 
FROM ble_packets 
ORDER BY id DESC 
LIMIT 20;
"#,
        },
        PredefinedQuery {
            id: 18,
            title: "18. iBeacon & Eddystone Beacon Logs",
            category: "Beacon Analysis",
            description: "BLE Beacons broadcasting location or marketing payloads",
            sql: r#"
SELECT 
    mac_address, 
    vendor_name, 
    last_adv_name, 
    max_rssi, 
    last_seen 
FROM ble_devices 
WHERE inferred_type LIKE '%Beacon%'
ORDER BY max_rssi DESC;
"#,
        },
        PredefinedQuery {
            id: 19,
            title: "19. HCI Adapter Packet Distribution (hci0 vs hci1)",
            category: "Adapter Health",
            description: "Compare packet capture counts across physical Bluetooth adapters",
            sql: r#"
SELECT 
    adapter_name, 
    COUNT(*) AS total_captured_packets 
FROM ble_packets 
GROUP BY adapter_name;
"#,
        },
        PredefinedQuery {
            id: 20,
            title: "20. Inferred Device Types Summary",
            category: "Device Inventory",
            description: "Summary count of all inferred device categories",
            sql: r#"
SELECT 
    inferred_type, 
    COUNT(*) AS device_count 
FROM ble_devices 
GROUP BY inferred_type 
ORDER BY device_count DESC;
"#,
        },
    ]
}
