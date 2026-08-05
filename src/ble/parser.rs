pub struct ParsedEir {
    pub local_name: Option<String>,
    pub manufacturer_id: Option<u16>,
    pub manufacturer_data: Option<Vec<u8>>,
    pub service_uuids: Vec<String>,
    pub appearance: Option<u16>,
    pub tx_power: Option<i8>,
}

pub fn parse_eir(raw: &[u8]) -> ParsedEir {
    let mut local_name = None;
    let mut manufacturer_id = None;
    let mut manufacturer_data = None;
    let mut service_uuids = Vec::new();
    let mut appearance = None;
    let mut tx_power = None;

    let mut pos = 0;
    while pos < raw.len() {
        let len = raw[pos] as usize;
        if len == 0 || pos + len >= raw.len() {
            break;
        }

        let ad_type = raw[pos + 1];
        let payload = &raw[pos + 2..pos + 1 + len];

        match ad_type {
            // Shortened or Complete Local Name
            0x08 | 0x09 => {
                if let Ok(name) = std::str::from_utf8(payload) {
                    local_name = Some(name.trim_matches('\0').to_string());
                }
            }
            // 16-bit Service UUIDs
            0x02 | 0x03 => {
                let mut i = 0;
                while i + 1 < payload.len() {
                    let uuid_val = u16::from_le_bytes([payload[i], payload[i + 1]]);
                    service_uuids.push(format!("0x{:04X}", uuid_val));
                    i += 2;
                }
            }
            // 128-bit Service UUIDs
            0x06 | 0x07 => {
                if payload.len() == 16 {
                    let mut bytes = payload.to_vec();
                    bytes.reverse(); // Reverse for canonical UUID string format
                    let uuid_str = format!(
                        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7],
                        bytes[8], bytes[9], bytes[10], bytes[11],
                        bytes[12], bytes[13], bytes[14], bytes[15]
                    );
                    service_uuids.push(uuid_str);
                }
            }
            // Tx Power Level
            0x0A => {
                if !payload.is_empty() {
                    tx_power = Some(payload[0] as i8);
                }
            }
            // Appearance
            0x19 => {
                if payload.len() >= 2 {
                    appearance = Some(u16::from_le_bytes([payload[0], payload[1]]));
                }
            }
            // Manufacturer Specific Data
            0xFF => {
                if payload.len() >= 2 {
                    let mfg_id = u16::from_le_bytes([payload[0], payload[1]]);
                    manufacturer_id = Some(mfg_id);
                    manufacturer_data = Some(payload[2..].to_vec());
                }
            }
            _ => {}
        }

        pos += len + 1;
    }

    ParsedEir {
        local_name,
        manufacturer_id,
        manufacturer_data,
        service_uuids,
        appearance,
        tx_power,
    }
}
