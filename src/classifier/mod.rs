pub mod oui_db;
pub mod rules;

pub use oui_db::OuiDatabase;
pub use rules::{infer_device_fingerprint, FingerprintResult};

use crate::ble::parser::ParsedEir;
use crate::ble::types::AddressType;

pub struct DeviceClassifier {
    oui_db: OuiDatabase,
}

impl DeviceClassifier {
    pub fn new() -> Self {
        Self {
            oui_db: OuiDatabase::new(),
        }
    }

    pub fn lookup_oui(&self, mac: &str) -> String {
        self.oui_db.lookup(mac)
    }

    pub fn classify_device(
        &self,
        mac: &str,
        addr_type: AddressType,
        eir: &ParsedEir,
        vendor_name: &str,
    ) -> FingerprintResult {
        infer_device_fingerprint(mac, addr_type, eir, vendor_name)
    }

    pub async fn update_oui_db(&self) -> crate::error::Result<usize> {
        self.oui_db.download_and_update().await
    }
}
