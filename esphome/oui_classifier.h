#pragma once

#include "esphome.h"
#include <vector>
#include <string>
#include <algorithm>

struct BleDeviceInfo {
    std::string mac;
    std::string name;
    int rssi;
    std::string vendor;
    std::string device_type;
};

class Esp32BleClassifier {
public:
    static std::string lookup_oui(const std::string& mac) {
        std::string upper_mac = mac;
        std::transform(upper_mac.begin(), upper_mac.end(), upper_mac.begin(), ::toupper);
        
        if (upper_mac.rfind("44:00:10", 0) == 0 || upper_mac.rfind("18:C0:4D", 0) == 0) return "Apple, Inc.";
        if (upper_mac.rfind("AC:37:43", 0) == 0 || upper_mac.rfind("00:07:AB", 0) == 0) return "Samsung Electronics";
        if (upper_mac.rfind("24:0A:C4", 0) == 0 || upper_mac.rfind("30:AE:A4", 0) == 0) return "Espressif Systems";
        if (upper_mac.rfind("70:B8:F6", 0) == 0 || upper_mac.rfind("8C:AA:B5", 0) == 0) return "Espressif Systems";
        if (upper_mac.rfind("F4:CE:36", 0) == 0 || upper_mac.rfind("E7:E4:01", 0) == 0) return "Nordic Semiconductor";
        if (upper_mac.rfind("D0:03:4B", 0) == 0) return "LG Electronics";
        if (upper_mac.rfind("00:1E:42", 0) == 0) return "Google LLC";
        
        return "Unknown Vendor";
    }

    static std::string infer_device_type(const std::string& mac, const std::string& name, int mfg_id, const std::string& vendor) {
        std::string name_lower = name;
        std::transform(name_lower.begin(), name_lower.end(), name_lower.begin(), ::tolower);

        if (vendor == "Espressif Systems" || name_lower.find("esp32") != std::string::npos) return "ESP32 / ESP8266";
        if (vendor == "Nordic Semiconductor" || name_lower.find("nrf") != std::string::npos) return "nRF Device (Nordic)";
        if (mfg_id == 0x004C || vendor == "Apple, Inc.") {
            if (name_lower.find("watch") != std::string::npos) return "Smartwatch";
            if (name_lower.find("airtag") != std::string::npos) return "AirTag / Tracker";
            return "iPhone / iPad";
        }
        if (name_lower.find("tile") != std::string::npos || name_lower.find("nut") != std::string::npos) return "AirTag / Tracker";
        if (mfg_id == 0x0075 || mfg_id == 0x00E0 || vendor == "Samsung Electronics") return "Android Phone";
        if (vendor == "LG Electronics" || name_lower.find("tv") != std::string::npos) return "Smart TV";
        if (name_lower.find("band") != std::string::npos || name_lower.find("fit") != std::string::npos) return "Smartwatch";

        return "Generic BLE Peripheral";
    }
};
