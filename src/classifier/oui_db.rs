use crate::error::Result;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{info, warn};

pub struct OuiDatabase {
    vendors: RwLock<HashMap<String, String>>,
}

impl OuiDatabase {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        // Built-in offline pre-packaged IEEE OUI database entries
        let builtins = vec![
            ("44:00:10", "Apple, Inc."),
            ("00:17:F2", "Apple, Inc."),
            ("00:1C:B3", "Apple, Inc."),
            ("00:1E:C2", "Apple, Inc."),
            ("00:23:12", "Apple, Inc."),
            ("00:25:00", "Apple, Inc."),
            ("00:26:08", "Apple, Inc."),
            ("00:26:BB", "Apple, Inc."),
            ("18:C0:4D", "Apple, Inc."),
            ("DC:A9:04", "Apple, Inc."),
            ("AC:37:43", "Samsung Electronics Co., Ltd."),
            ("00:07:AB", "Samsung Electronics Co., Ltd."),
            ("00:12:FB", "Samsung Electronics Co., Ltd."),
            ("00:15:99", "Samsung Electronics Co., Ltd."),
            ("00:1A:8A", "Samsung Electronics Co., Ltd."),
            ("00:E0:4C", "Realtek Semiconductor Corp."),
            ("00:1E:42", "Google LLC"),
            ("3C:5A:B4", "Google LLC"),
            ("94:EB:CD", "Google LLC"),
            ("24:0A:C4", "Espressif Systems (Shanghai) CO., LTD"),
            ("30:AE:A4", "Espressif Systems (Shanghai) CO., LTD"),
            ("84:0D:8E", "Espressif Systems (Shanghai) CO., LTD"),
            ("70:B8:F6", "Espressif Systems (Shanghai) CO., LTD"),
            ("8C:AA:B5", "Espressif Systems (Shanghai) CO., LTD"),
            ("DC:DA:0C", "Espressif Systems (Shanghai) CO., LTD"),
            ("F4:CE:36", "Nordic Semiconductor ASA"),
            ("E7:E4:01", "Nordic Semiconductor ASA"),
            ("00:09:A7", "Nordic Semiconductor ASA"),
            ("D0:03:4B", "LG Electronics (Mobile Communications)"),
            ("00:0B:E1", "LG Electronics"),
            ("00:1A:7D", "Sena Technologies, Inc."), // Sena UD-100 HCI dongles
            ("00:50:F2", "Microsoft Corporation"),
            ("00:15:5D", "Microsoft Corporation"),
            ("00:04:4B", "NVIDIA Corporation"),
            ("00:09:2D", "HTC Corporation"),
            ("00:0E:6D", "Garmin International"),
            ("00:1C:7B", "Fitbit, Inc."),
            ("00:1D:BA", "Sony Corporation"),
            ("00:01:4A", "Sony Interactive Entertainment Inc."),
            ("00:04:20", "Slim Devices, Inc. (Logitech)"),
            ("00:0D:4B", "Roku, Inc."),
            ("28:ED:E0", "Raspberry Pi Trading Ltd"),
            ("B8:27:EB", "Raspberry Pi Foundation"),
            ("DC:A6:32", "Raspberry Pi Trading Ltd"),
            ("E4:5F:01", "Raspberry Pi Trading Ltd"),
        ];

        for (prefix, vendor) in builtins {
            map.insert(prefix.to_uppercase(), vendor.to_string());
        }

        Self {
            vendors: RwLock::new(map),
        }
    }

    pub fn lookup(&self, mac: &str) -> String {
        let clean_mac = mac.replace('-', ":").to_uppercase();
        if clean_mac.len() >= 8 {
            let prefix = &clean_mac[0..8];
            if let Ok(map) = self.vendors.read() {
                if let Some(vendor) = map.get(prefix) {
                    return vendor.clone();
                }
            }
        }
        "Unknown Vendor".to_string()
    }

    pub async fn download_and_update(&self) -> Result<usize> {
        info!("Fetching latest IEEE OUI database mirror...");
        let url = "https://raw.githubusercontent.com/wireshark/wireshark/master/manuf";

        match reqwest::get(url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let body = resp.text().await?;
                    let mut count = 0;
                    if let Ok(mut map) = self.vendors.write() {
                        for line in body.lines() {
                            if line.starts_with('#') || line.trim().is_empty() {
                                continue;
                            }
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let prefix = parts[0].replace('-', ":").to_uppercase();
                                if prefix.len() == 8 {
                                    let vendor = parts[1..].join(" ");
                                    map.insert(prefix, vendor);
                                    count += 1;
                                }
                            }
                        }
                    }
                    info!("Successfully updated IEEE OUI database with {} vendors.", count);
                    return Ok(count);
                }
            }
            Err(e) => {
                warn!("Could not fetch remote IEEE OUI database (Offline mode retained): {}", e);
            }
        }

        Ok(0)
    }
}
