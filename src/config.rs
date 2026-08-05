use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub hci_adapters: Vec<String>,
    pub camera_device: String,
    pub camera_enabled: bool,
    pub http_port: u16,
    pub db_path: PathBuf,
    pub storage_dir: PathBuf,
    pub video_dir: PathBuf,
    pub motion_threshold: f64,
    pub pre_roll_secs: u64,
    pub post_roll_secs: u64,
    pub auto_download_oui: bool,
    
    // MQTT Options
    pub mqtt_enabled: bool,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_topic: String,
    pub mqtt_user: Option<String>,
    pub mqtt_pass: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let storage_dir = PathBuf::from("storage");
        let video_dir = storage_dir.join("videos");
        let db_path = storage_dir.join("sniffer.db");

        Self {
            hci_adapters: vec!["hci0".to_string()],
            camera_device: "/dev/video0".to_string(),
            camera_enabled: true,
            http_port: 8080,
            db_path,
            storage_dir,
            video_dir,
            motion_threshold: 15.0,
            pre_roll_secs: 10,
            post_roll_secs: 10,
            auto_download_oui: true,

            mqtt_enabled: false,
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_topic: "homeassistant/rpi_ble_opsec".to_string(),
            mqtt_user: None,
            mqtt_pass: None,
        }
    }
}

impl Config {
    pub fn parse_args() -> Self {
        let mut config = Config::default();
        let args: Vec<String> = env::args().collect();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--hci" => {
                    if i + 1 < args.len() {
                        config.hci_adapters = args[i + 1]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                        i += 1;
                    }
                }
                "--camera" => {
                    if i + 1 < args.len() {
                        config.camera_device = args[i + 1].clone();
                        i += 1;
                    }
                }
                "--no-camera" => {
                    config.camera_enabled = false;
                }
                "--http-port" | "-p" => {
                    if i + 1 < args.len() {
                        if let Ok(port) = args[i + 1].parse() {
                            config.http_port = port;
                        }
                        i += 1;
                    }
                }
                "--mqtt-host" => {
                    if i + 1 < args.len() {
                        config.mqtt_host = args[i + 1].clone();
                        config.mqtt_enabled = true;
                        i += 1;
                    }
                }
                "--mqtt-port" => {
                    if i + 1 < args.len() {
                        if let Ok(port) = args[i + 1].parse() {
                            config.mqtt_port = port;
                        }
                        i += 1;
                    }
                }
                "--mqtt-topic" => {
                    if i + 1 < args.len() {
                        config.mqtt_topic = args[i + 1].clone();
                        i += 1;
                    }
                }
                "--mqtt-user" => {
                    if i + 1 < args.len() {
                        config.mqtt_user = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--mqtt-pass" => {
                    if i + 1 < args.len() {
                        config.mqtt_pass = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--db" => {
                    if i + 1 < args.len() {
                        config.db_path = PathBuf::from(&args[i + 1]);
                        i += 1;
                    }
                }
                "--motion-threshold" => {
                    if i + 1 < args.len() {
                        if let Ok(t) = args[i + 1].parse() {
                            config.motion_threshold = t;
                        }
                        i += 1;
                    }
                }
                "--no-oui-download" => {
                    config.auto_download_oui = false;
                }
                "--help" | "-h" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                _ => {}
            }
            i += 1;
        }

        config
    }

    fn print_help() {
        println!("Raspberry Pi Portable BLE OPSEC Sniffer & Motion Correlator");
        println!();
        println!("USAGE:");
        println!("  rpi-portable-ble-opsec-sniffer [FLAGS] [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("  --hci <adapters>      Comma-separated list of HCI adapters (default: hci0)");
        println!("  --camera <dev>        V4L2 camera path (default: /dev/video0)");
        println!("  --no-camera           Disable camera video motion recording");
        println!("  --http-port <port>    Web interface HTTP port (default: 8080)");
        println!("  --mqtt-host <host>    MQTT Broker Host IP (Enables MQTT publishing)");
        println!("  --mqtt-port <port>    MQTT Broker Port (default: 1883)");
        println!("  --mqtt-topic <topic>  MQTT Topic prefix (default: homeassistant/rpi_ble_opsec)");
        println!("  --mqtt-user <user>    MQTT Username");
        println!("  --mqtt-pass <pass>    MQTT Password");
        println!("  --db <path>           SQLite DB file path (default: storage/sniffer.db)");
        println!("  --motion-threshold <val> Sensitivity threshold for motion (default: 15.0)");
        println!("  --no-oui-download     Disable automatic IEEE OUI database updates");
        println!("  -h, --help            Show this help message");
    }
}
