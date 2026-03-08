use std::fs;
use std::time::Duration;
use yaml_rust2::YamlLoader;

#[derive(Clone)]
pub struct DimmerConfig {
    pub use_dimmer: bool,
    pub timeout: Duration,
    pub backlight: String,
    pub input_devices: Vec<String>,
    pub turn_backlight_off: bool, // TODO implement
    pub dimmed_brightness: u8,
    pub full_brightness: u8,
}

#[derive(Clone)]
pub struct MqttConfig {
    pub use_mqtt: bool,
    pub server_address: String,
    pub server_port: u16,
    pub connection_attempts: u64,
    pub auth_username: String,
    pub auth_password: String,
    pub discovery_topic_prefix: String,
    pub app_topic_prefix: String,
    pub device_id: String,
    pub device_name: String,
}

#[derive(Clone)]
pub struct Config {
    pub dimmer: DimmerConfig,
    pub mqtt: MqttConfig,
}

impl Config {
    pub fn new(file_name: &str) -> Self {
        let file_content = fs::read_to_string(file_name).expect("Unable to read config file");
        let configs =
            YamlLoader::load_from_str(&file_content).expect("Unable to parse config file");
        let config = &configs[0];
        let dimmer = DimmerConfig {
            use_dimmer: config["dimmer"]["use_dimmer"].as_bool().unwrap_or(true),
            timeout: Duration::new(
                config["dimmer"]["timeout_sec"].as_i64().unwrap_or(30) as u64,
                0,
            ),
            backlight: config["dimmer"]["backlight"]
                .as_str()
                .unwrap_or("/sys/class/backlight/rpi-backlight")
                .to_string(),
            input_devices: config["dimmer"]["input_devices"]
                .as_vec()
                .map(|devices| devices.to_vec())
                .unwrap_or_default()
                .iter()
                .map(|device| {
                    device
                        .as_str()
                        .expect("Unsupported value for input_devices")
                        .to_string()
                })
                .collect::<Vec<String>>(),
            turn_backlight_off: config["dimmer"]["turn_backlight_off"]
                .as_bool()
                .unwrap_or(false),
            dimmed_brightness: config["dimmer"]["dimmed_brightness"].as_i64().unwrap_or(30) as u8,
            full_brightness: config["dimmer"]["full_brightness"].as_i64().unwrap_or(255) as u8,
        };
        let mqtt = MqttConfig {
            use_mqtt: config["mqtt"]["use_mqtt"].as_bool().unwrap_or(false),
            server_address: config["mqtt"]["server_address"]
                .as_str()
                .unwrap_or("127.0.0.1")
                .to_string(),
            server_port: config["mqtt"]["server_port"].as_i64().unwrap_or(1883) as u16,
            auth_username: config["mqtt"]["auth_username"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            auth_password: config["mqtt"]["auth_password"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            connection_attempts: config["mqtt"]["connection_attempts"].as_i64().unwrap_or(10)
                as u64,
            discovery_topic_prefix: config["mqtt"]["discovery_topic_prefix"]
                .as_str()
                .unwrap_or("homeassistant")
                .to_string(),
            app_topic_prefix: config["mqtt"]["app_topic_prefix"]
                .as_str()
                .unwrap_or("pi_ts_control")
                .to_string(),
            device_id: config["mqtt"]["device_id"]
                .as_str()
                .unwrap_or("pi_ts_control")
                .to_string(),
            device_name: config["mqtt"]["device_name"]
                .as_str()
                .unwrap_or("Pi Touchscreen")
                .to_string(),
        };
        Self { dimmer, mqtt }
    }

    // TODO implement writing back to config file
    /*pub fn apply_state(&mut self, state: &state::State) -> bool {
        let mut config_changed = false;
        if state.backlight_active_dimmed != self.dimmer.turn_backlight_off {
            self.dimmer.turn_backlight_off = state.backlight_active_dimmed;
            config_changed = true;
        }
        if state.brightness_dimmed != self.dimmer.dimmed_brightness {
            self.dimmer.dimmed_brightness = state.brightness_dimmed;
            config_changed = true;
        }
        if state.brightness_full != self.dimmer.full_brightness {
            self.dimmer.full_brightness = state.brightness_full;
            config_changed = true;
        }
        if state.use_dimmer != self.dimmer.use_dimmer {
            self.dimmer.use_dimmer = state.use_dimmer;
            config_changed = true;
        }
        if state.timeout != self.dimmer.timeout {
            self.dimmer.timeout = state.timeout;
            config_changed = true;
        }
        config_changed
    }*/
}
