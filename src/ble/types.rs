use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressType {
    Public,
    RandomStatic,
    RandomPrivateResolvable,    // RPA
    RandomPrivateNonResolvable, // NRPA
    Unknown,
}

impl AddressType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => AddressType::Public,
            0x01 => AddressType::RandomStatic,
            0x02 => AddressType::RandomPrivateResolvable,
            0x03 => AddressType::RandomPrivateNonResolvable,
            _ => AddressType::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AddressType::Public => "Public",
            AddressType::RandomStatic => "Random Static",
            AddressType::RandomPrivateResolvable => "RPA (Resolvable Private)",
            AddressType::RandomPrivateNonResolvable => "NRPA (Non-Resolvable Private)",
            AddressType::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlePacket {
    pub timestamp: String, // ISO 8601 string
    pub timestamp_epoch: i64,
    pub adapter_name: String,
    pub mac_address: String,
    pub address_type: AddressType,
    pub rssi: i8,
    pub raw_eir: Vec<u8>,
    pub local_name: Option<String>,
    pub manufacturer_id: Option<u16>,
    pub manufacturer_data: Option<Vec<u8>>,
    pub service_uuids: Vec<String>,
    pub appearance: Option<u16>,
    pub tx_power: Option<i8>,
    pub is_extended: bool,
    pub inferred_device_type: String,
    pub vendor_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleDeviceSummary {
    pub mac_address: String,
    pub address_type: String,
    pub vendor_name: String,
    pub inferred_type: String,
    pub first_seen: String,
    pub last_seen: String,
    pub max_rssi: i8,
    pub last_rssi: i8,
    pub total_packets: u64,
    pub last_adv_name: Option<String>,
    pub last_raw_eir_hex: String,
}
