use crate::ble::parser::ParsedEir;
use crate::ble::types::AddressType;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DeviceType {
    IPhone,
    IPad,
    AirPods,
    AirTagTracker,
    AppleWatch,
    MacComputer,
    AndroidPhone,
    SamsungDevice,
    GooglePixelDevice,
    Smartwatch,
    SmartTv,
    WindowsPc,
    Esp32,
    NrfDevice,
    Beacon,
    SmartHomeIoT,
    AudioHeadphones,
    HidInputDevice,
    MedicalDevice,
    Automotive,
    GameConsole,
    DroneUav,
    EvCharger,
    SmartScaleFitness,
    SecurityCam,
    GenericPeripheral,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::IPhone => "iPhone",
            DeviceType::IPad => "iPad",
            DeviceType::AirPods => "AirPods / Beats",
            DeviceType::AirTagTracker => "AirTag / FindMy Tracker",
            DeviceType::AppleWatch => "Apple Watch",
            DeviceType::MacComputer => "Mac / MacBook",
            DeviceType::AndroidPhone => "Android Phone",
            DeviceType::SamsungDevice => "Samsung Galaxy Device",
            DeviceType::GooglePixelDevice => "Google Pixel / FastPair",
            DeviceType::Smartwatch => "Smartwatch / Wearable",
            DeviceType::SmartTv => "Smart TV / Media Player",
            DeviceType::WindowsPc => "Windows PC (Swift Pair)",
            DeviceType::Esp32 => "ESP32 / ESP8266",
            DeviceType::NrfDevice => "nRF Device (Nordic)",
            DeviceType::Beacon => "iBeacon / Eddystone",
            DeviceType::SmartHomeIoT => "Smart Home / IoT",
            DeviceType::AudioHeadphones => "Audio / Headphones",
            DeviceType::HidInputDevice => "HID Keyboard/Mouse",
            DeviceType::MedicalDevice => "Medical / Health Device",
            DeviceType::Automotive => "Automotive / Car BLE",
            DeviceType::GameConsole => "Game Console (PS5/Xbox/Switch)",
            DeviceType::DroneUav => "Drone / UAV Remote ID",
            DeviceType::EvCharger => "EV Charger / Power Station",
            DeviceType::SmartScaleFitness => "Smart Scale / Fitness",
            DeviceType::SecurityCam => "Security Camera / Alarm",
            DeviceType::GenericPeripheral => "Generic BLE Peripheral",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FingerprintResult {
    pub device_type: DeviceType,
    pub vendor_name: String,
    pub is_randomized_mac: bool,
    pub fingerprint_confidence: &'static str, // "High (Payload)", "Medium (Appearance)", "OUI"
}

/// Advanced Multi-Layer BLE Fingerprinting Engine
/// Correctly infers Brand, Manufacturer, and Device Category even on Randomized RPA/NRPA MACs!
pub fn infer_device_fingerprint(
    _mac: &str,
    addr_type: AddressType,
    eir: &ParsedEir,
    oui_vendor: &str,
) -> FingerprintResult {
    let name_lower = eir.local_name.as_deref().unwrap_or("").to_lowercase();
    let is_random_mac = addr_type == AddressType::RandomPrivateResolvable
        || addr_type == AddressType::RandomPrivateNonResolvable;

    // Default vendor from OUI unless overridden by Manufacturer Payload
    let mut vendor = if oui_vendor != "Unknown Vendor" {
        oui_vendor.to_string()
    } else {
        "Unknown Vendor".to_string()
    };

    // =========================================================================
    // LAYER 1: Manufacturer Specific Data (0xFF) Deep Inspection (Bypasses RPA)
    // =========================================================================
    if let Some(mfg_id) = eir.manufacturer_id {
        let mfg_data = eir.manufacturer_data.as_deref().unwrap_or(&[]);

        match mfg_id {
            // -----------------------------------------------------------------
            // Apple, Inc. (0x004C) - Continuity, FindMy, AirDrop, AirPods
            // -----------------------------------------------------------------
            0x004C => {
                vendor = "Apple, Inc.".to_string();
                if !mfg_data.is_empty() {
                    match mfg_data[0] {
                        // 0x12 / 0x16 = FindMy Network / AirTag Tracker Payload
                        0x12 | 0x16 => {
                            return FingerprintResult {
                                device_type: DeviceType::AirTagTracker,
                                vendor_name: vendor,
                                is_randomized_mac: is_random_mac,
                                fingerprint_confidence: "High (Apple FindMy Payload)",
                            };
                        }
                        // 0x07 = AirPods / Beats Headphones
                        0x07 => {
                            return FingerprintResult {
                                device_type: DeviceType::AirPods,
                                vendor_name: vendor,
                                is_randomized_mac: is_random_mac,
                                fingerprint_confidence: "High (Apple AirPods Payload)",
                            };
                        }
                        // 0x02 = iBeacon
                        0x02 => {
                            return FingerprintResult {
                                device_type: DeviceType::Beacon,
                                vendor_name: vendor,
                                is_randomized_mac: is_random_mac,
                                fingerprint_confidence: "High (Apple iBeacon Payload)",
                            };
                        }
                        // 0x10 = Nearby Action / Continuity
                        0x10 => {
                            if name_lower.contains("ipad") {
                                return FingerprintResult {
                                    device_type: DeviceType::IPad,
                                    vendor_name: vendor,
                                    is_randomized_mac: is_random_mac,
                                    fingerprint_confidence: "High (Apple Continuity Payload)",
                                };
                            }
                            if name_lower.contains("watch") {
                                return FingerprintResult {
                                    device_type: DeviceType::AppleWatch,
                                    vendor_name: vendor,
                                    is_randomized_mac: is_random_mac,
                                    fingerprint_confidence: "High (Apple Watch Payload)",
                                };
                            }
                            if name_lower.contains("macbook") || name_lower.contains("mac") {
                                return FingerprintResult {
                                    device_type: DeviceType::MacComputer,
                                    vendor_name: vendor,
                                    is_randomized_mac: is_random_mac,
                                    fingerprint_confidence: "High (Apple Mac Payload)",
                                };
                            }
                            return FingerprintResult {
                                device_type: DeviceType::IPhone,
                                vendor_name: vendor,
                                is_randomized_mac: is_random_mac,
                                fingerprint_confidence: "High (Apple iPhone Payload)",
                            };
                        }
                        _ => {}
                    }
                }
                return FingerprintResult {
                    device_type: DeviceType::IPhone,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "High (Apple Mfg ID 0x004C)",
                };
            }

            // -----------------------------------------------------------------
            // Microsoft Corporation (0x0006) - Swift Pair / Windows Beacon
            // -----------------------------------------------------------------
            0x0006 => {
                vendor = "Microsoft Corporation".to_string();
                return FingerprintResult {
                    device_type: DeviceType::WindowsPc,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "High (Microsoft Swift Pair Payload)",
                };
            }

            // -----------------------------------------------------------------
            // Samsung Electronics (0x0075) - SmartThings & Galaxy
            // -----------------------------------------------------------------
            0x0075 => {
                vendor = "Samsung Electronics".to_string();
                if name_lower.contains("tv") {
                    return FingerprintResult {
                        device_type: DeviceType::SmartTv,
                        vendor_name: vendor,
                        is_randomized_mac: is_random_mac,
                        fingerprint_confidence: "High (Samsung Smart TV Payload)",
                    };
                }
                return FingerprintResult {
                    device_type: DeviceType::SamsungDevice,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "High (Samsung Mfg ID 0x0075)",
                };
            }

            // -----------------------------------------------------------------
            // Google LLC (0x00E0) - FastPair & Nearby Share
            // -----------------------------------------------------------------
            0x00E0 => {
                vendor = "Google LLC".to_string();
                return FingerprintResult {
                    device_type: DeviceType::GooglePixelDevice,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "High (Google FastPair Payload)",
                };
            }

            // Espressif ESP32 (0x02E5 / 0x07D7)
            0x02E5 | 0x07D7 => {
                vendor = "Espressif Systems".to_string();
                return FingerprintResult {
                    device_type: DeviceType::Esp32,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "High (Espressif Payload)",
                };
            }

            // Nordic Semiconductor (0x0059)
            0x0059 => {
                vendor = "Nordic Semiconductor ASA".to_string();
                return FingerprintResult {
                    device_type: DeviceType::NrfDevice,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "High (Nordic Semi Payload)",
                };
            }

            _ => {}
        }
    }

    // =========================================================================
    // LAYER 2: Service UUID Inspection (Google FastPair 0xFE2C, NUS, DFU, etc.)
    // =========================================================================
    for uuid in &eir.service_uuids {
        let u = uuid.to_uppercase();
        if u.contains("FE2C") || u.contains("FE9F") {
            if vendor == "Unknown Vendor" {
                vendor = "Google LLC".to_string();
            }
            return FingerprintResult {
                device_type: DeviceType::GooglePixelDevice,
                vendor_name: vendor,
                is_randomized_mac: is_random_mac,
                fingerprint_confidence: "High (Google FastPair Service)",
            };
        }
        if u.contains("FEAA") || u.contains("FEED") || u.contains("FE33") {
            return FingerprintResult {
                device_type: DeviceType::AirTagTracker,
                vendor_name: vendor,
                is_randomized_mac: is_random_mac,
                fingerprint_confidence: "High (Tracker / Beacon Service)",
            };
        }
        if u.contains("6E400001") || u.contains("FE59") {
            if vendor == "Unknown Vendor" {
                vendor = "Nordic Semiconductor ASA".to_string();
            }
            return FingerprintResult {
                device_type: DeviceType::NrfDevice,
                vendor_name: vendor,
                is_randomized_mac: is_random_mac,
                fingerprint_confidence: "High (Nordic NUS/DFU Service)",
            };
        }
        if u.contains("180D") || u.contains("1809") || u.contains("1808") || u.contains("1810") {
            return FingerprintResult {
                device_type: DeviceType::MedicalDevice,
                vendor_name: vendor,
                is_randomized_mac: is_random_mac,
                fingerprint_confidence: "High (Medical Service UUID)",
            };
        }
    }

    // =========================================================================
    // LAYER 3: Bluetooth SIG Appearance Code Heuristics
    // =========================================================================
    if let Some(app) = eir.appearance {
        match app {
            0x0040..=0x007F => {
                return FingerprintResult {
                    device_type: DeviceType::AndroidPhone,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "Medium (Appearance: Phone)",
                };
            }
            0x00C0..=0x00FF => {
                return FingerprintResult {
                    device_type: DeviceType::Smartwatch,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "Medium (Appearance: Watch)",
                };
            }
            0x0140..=0x017F => {
                return FingerprintResult {
                    device_type: DeviceType::SmartTv,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "Medium (Appearance: Display)",
                };
            }
            0x03C0..=0x03CF => {
                return FingerprintResult {
                    device_type: DeviceType::HidInputDevice,
                    vendor_name: vendor,
                    is_randomized_mac: is_random_mac,
                    fingerprint_confidence: "Medium (Appearance: HID)",
                };
            }
            _ => {}
        }
    }

    // =========================================================================
    // LAYER 4: Local Name & Vendor Keyword Fallback Matching
    // =========================================================================
    let vendor_lower = vendor.to_lowercase();

    if vendor_lower.contains("espressif") || name_lower.contains("esp32") || name_lower.contains("esp-") {
        return FingerprintResult {
            device_type: DeviceType::Esp32,
            vendor_name: "Espressif Systems".to_string(),
            is_randomized_mac: is_random_mac,
            fingerprint_confidence: "Medium (Name / OUI)",
        };
    }

    if vendor_lower.contains("nordic") || name_lower.contains("nrf") {
        return FingerprintResult {
            device_type: DeviceType::NrfDevice,
            vendor_name: "Nordic Semiconductor ASA".to_string(),
            is_randomized_mac: is_random_mac,
            fingerprint_confidence: "Medium (Name / OUI)",
        };
    }

    if name_lower.contains("iphone") {
        return FingerprintResult {
            device_type: DeviceType::IPhone,
            vendor_name: "Apple, Inc.".to_string(),
            is_randomized_mac: is_random_mac,
            fingerprint_confidence: "Medium (Local Name)",
        };
    }

    if name_lower.contains("galaxy") || vendor_lower.contains("samsung") {
        return FingerprintResult {
            device_type: DeviceType::SamsungDevice,
            vendor_name: "Samsung Electronics".to_string(),
            is_randomized_mac: is_random_mac,
            fingerprint_confidence: "Medium (Name / OUI)",
        };
    }

    if name_lower.contains("pixel") || vendor_lower.contains("google") {
        return FingerprintResult {
            device_type: DeviceType::GooglePixelDevice,
            vendor_name: "Google LLC".to_string(),
            is_randomized_mac: is_random_mac,
            fingerprint_confidence: "Medium (Name / OUI)",
        };
    }

    FingerprintResult {
        device_type: DeviceType::GenericPeripheral,
        vendor_name: vendor,
        is_randomized_mac: is_random_mac,
        fingerprint_confidence: if is_random_mac {
            "Low (RPA Random MAC)"
        } else {
            "OUI Lookup"
        },
    }
}
