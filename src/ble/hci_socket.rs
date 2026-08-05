use crate::ble::parser::parse_eir;
use crate::ble::types::{AddressType, BlePacket};
use crate::classifier::DeviceClassifier;
use crate::error::Result;

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

// Linux AF_BLUETOOTH constants
const AF_BLUETOOTH: i32 = 31;
const SOCK_RAW: i32 = 3;
const BTPROTO_HCI: i32 = 1;

pub struct HciScanner {
    adapter_name: String,
    adapter_index: u16,
    classifier: Arc<DeviceClassifier>,
}

impl HciScanner {
    pub fn new(adapter_name: &str, classifier: Arc<DeviceClassifier>) -> Self {
        let index = if adapter_name.starts_with("hci") {
            adapter_name[3..].parse::<u16>().unwrap_or(0)
        } else {
            0
        };

        Self {
            adapter_name: adapter_name.to_string(),
            adapter_index: index,
            classifier,
        }
    }

    pub async fn start_scan_loop(&self, tx: mpsc::Sender<BlePacket>) -> Result<()> {
        info!("Starting 24/7 BLE scan loop on HCI adapter '{}'...", self.adapter_name);

        // Attempt to open raw HCI socket on Linux
        let socket_fd = unsafe { libc::socket(AF_BLUETOOTH, SOCK_RAW, BTPROTO_HCI) };

        if socket_fd < 0 {
            warn!(
                "Could not open AF_BLUETOOTH HCI raw socket on '{}' (Requires root/CAP_NET_RAW). Enabling HCI Simulation Mode for testing & UI preview.",
                self.adapter_name
            );
            return self.run_simulation_loop(tx).await;
        }

        // Bind socket to adapter index
        let mut sockaddr: [u8; 6] = [0; 6];
        // family AF_BLUETOOTH (u16 LE = 31)
        sockaddr[0] = (AF_BLUETOOTH & 0xFF) as u8;
        sockaddr[1] = ((AF_BLUETOOTH >> 8) & 0xFF) as u8;
        // dev index
        sockaddr[2] = (self.adapter_index & 0xFF) as u8;
        sockaddr[3] = ((self.adapter_index >> 8) & 0xFF) as u8;
        // channel (0 = HCI_CHANNEL_RAW)
        sockaddr[4] = 0;
        sockaddr[5] = 0;

        let res = unsafe {
            libc::bind(
                socket_fd,
                sockaddr.as_ptr() as *const libc::sockaddr,
                sockaddr.len() as u32,
            )
        };

        if res < 0 {
            warn!("Failed to bind to HCI adapter {}. Falling back to HCI simulation.", self.adapter_name);
            unsafe { libc::close(socket_fd) };
            return self.run_simulation_loop(tx).await;
        }

        info!("Successfully bound raw HCI socket on {}", self.adapter_name);

        // Buffer for receiving HCI events (LE Advertising Report: Event code 0x3E / Sub-event 0x02 or Extended 0x0D)
        let mut buf = [0u8; 1024];

        loop {
            let n = unsafe {
                libc::recv(
                    socket_fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    0,
                )
            };

            if n > 0 {
                let packet_bytes = &buf[..n as usize];
                if let Some(packet) = self.parse_hci_packet(packet_bytes) {
                    if tx.send(packet).await.is_err() {
                        break;
                    }
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        unsafe { libc::close(socket_fd) };
        Ok(())
    }

    fn parse_hci_packet(&self, raw: &[u8]) -> Option<BlePacket> {
        // Basic LE Advertising Packet parsing
        // HCI Event packet starts with 0x04 (HCI Event Packet)
        if raw.len() < 14 || raw[0] != 0x04 || raw[1] != 0x3E {
            return None;
        }

        let sub_event = raw[3];
        // 0x02 = LE Advertising Report, 0x0D = LE Extended Advertising Report (BT 5.0/5.4/6.0)
        let is_extended = sub_event == 0x0D;

        if sub_event != 0x02 && !is_extended {
            return None;
        }

        let num_reports = raw[4];
        if num_reports == 0 {
            return None;
        }

        let addr_type_byte = raw[5];
        let address_type = AddressType::from_u8(addr_type_byte);

        // Extract 6-byte BD_ADDR (LE order)
        let mac_addr = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            raw[11], raw[10], raw[9], raw[8], raw[7], raw[6]
        );

        let data_len = raw[12] as usize;
        if raw.len() < 13 + data_len {
            return None;
        }

        let eir_payload = &raw[13..13 + data_len];
        let rssi_idx = 13 + data_len;
        let rssi = if raw.len() > rssi_idx {
            raw[rssi_idx] as i8
        } else {
            -70
        };

        let parsed = parse_eir(eir_payload);

        let oui_vendor = self.classifier.lookup_oui(&mac_addr);
        let fp = self.classifier.classify_device(
            &mac_addr,
            address_type,
            &parsed,
            &oui_vendor,
        );

        let now = Utc::now();

        Some(BlePacket {
            timestamp: now.to_rfc3339(),
            timestamp_epoch: now.timestamp(),
            adapter_name: self.adapter_name.clone(),
            mac_address: mac_addr,
            address_type,
            rssi,
            raw_eir: eir_payload.to_vec(),
            local_name: parsed.local_name,
            manufacturer_id: parsed.manufacturer_id,
            manufacturer_data: parsed.manufacturer_data,
            service_uuids: parsed.service_uuids,
            appearance: parsed.appearance,
            tx_power: parsed.tx_power,
            is_extended,
            inferred_device_type: fp.device_type.as_str().to_string(),
            vendor_name: fp.vendor_name,
        })
    }

    /// Generates realistic synthetic BLE packets when physical HCI sockets are unavailable (e.g. non-root or VM testing).
    async fn run_simulation_loop(&self, tx: mpsc::Sender<BlePacket>) -> Result<()> {
        use rand::Rng;

        let mock_devices = vec![
            ("44:00:10:A1:B2:C3", AddressType::RandomPrivateResolvable, "iPhone 15 Pro", "Apple, Inc.", "iPhone", vec![0x02, 0x01, 0x06, 0x11, 0xFF, 0x4C, 0x00, 0x10, 0x05, 0x03, 0x1C, 0xA1, 0xB2, 0xC3]),
            ("F4:CE:36:11:22:33", AddressType::Public, "nRF52840-Sensor", "Nordic Semiconductor", "nRF Device", vec![0x02, 0x01, 0x06, 0x09, 0x09, 0x6E, 0x52, 0x46, 0x35, 0x32, 0x38, 0x34, 0x30]),
            ("24:0A:C4:55:66:77", AddressType::Public, "ESP32_Beacon", "Espressif Inc.", "ESP32", vec![0x02, 0x01, 0x06, 0x0C, 0x09, 0x45, 0x53, 0x50, 0x33, 0x32, 0x5F, 0x42, 0x65, 0x61, 0x63, 0x6F, 0x6E]),
            ("AC:37:43:88:99:00", AddressType::RandomPrivateResolvable, "Galaxy S24 Ultra", "Samsung Electronics", "Android Phone", vec![0x02, 0x01, 0x06, 0x0B, 0xFF, 0x75, 0x00, 0x42, 0x04, 0x01, 0x20, 0x30]),
            ("70:B8:F6:12:34:56", AddressType::Public, "AirTag_Item_Tracker", "Apple, Inc.", "AirTag / Tracker", vec![0x02, 0x01, 0x06, 0x12, 0xFF, 0x4C, 0x00, 0x12, 0x19, 0x10, 0x00, 0x01, 0x02, 0x03]),
            ("18:C0:4D:99:88:77", AddressType::Public, "Apple Watch Ultra", "Apple, Inc.", "Smartwatch", vec![0x02, 0x01, 0x06, 0x08, 0xFF, 0x4C, 0x00, 0x07, 0x19, 0x03, 0xC0]),
            ("D0:03:4B:44:55:66", AddressType::Public, "LG OLED TV 65", "LG Electronics", "Smart TV", vec![0x02, 0x01, 0x06, 0x0B, 0x09, 0x4C, 0x47, 0x20, 0x4F, 0x4C, 0x45, 0x44, 0x20, 0x54, 0x56]),
            ("00:1A:7D:77:88:99", AddressType::Public, "Sena UD100 Node", "Sena Technologies", "Generic BLE Peripheral", vec![0x02, 0x01, 0x06]),
        ];

        loop {
            let (mac, addr_type, name, vendor, _dev_type, eir_raw) = {
                let mut rng = rand::thread_rng();
                &mock_devices[rng.gen_range(0..mock_devices.len())]
            };
            let rssi: i8 = {
                let mut rng = rand::thread_rng();
                rng.gen_range(-95..-35)
            };

            let parsed = parse_eir(&eir_raw);
            let fp = self.classifier.classify_device(
                mac,
                *addr_type,
                &parsed,
                vendor,
            );
            let now = Utc::now();

            let packet = BlePacket {
                timestamp: now.to_rfc3339(),
                timestamp_epoch: now.timestamp(),
                adapter_name: self.adapter_name.clone(),
                mac_address: mac.to_string(),
                address_type: *addr_type,
                rssi,
                raw_eir: eir_raw.to_vec(),
                local_name: parsed.local_name.or(Some(name.to_string())),
                manufacturer_id: parsed.manufacturer_id,
                manufacturer_data: parsed.manufacturer_data,
                service_uuids: parsed.service_uuids,
                appearance: parsed.appearance,
                tx_power: parsed.tx_power,
                is_extended: false,
                inferred_device_type: fp.device_type.as_str().to_string(),
                vendor_name: fp.vendor_name,
            };

            if tx.send(packet).await.is_err() {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Ok(())
    }
}
