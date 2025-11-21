use std::fs;
use std::time::Duration;
use yaml_rust2::YamlLoader;

pub struct Dimmer {
    pub timeout: Duration,
    pub backlight: String,
    pub input_devices: Vec<String>,
    pub dimmed_brightness: u8,
    pub full_brightness: u8,
}

pub struct Config {
    pub dimmer: Dimmer,
}

impl Config {
    pub fn new(file_name: &str) -> Self {
        let file_content = fs::read_to_string(file_name).expect("Unable to read config file");
        let configs = YamlLoader::load_from_str(&file_content).expect("Unable to parse config file");
        let config = &configs[0];
        let dimmer = Dimmer {
            timeout: Duration::new(config["dimmer"]["timeout_sec"].as_i64().unwrap_or(30) as u64, 0),
            backlight: config["dimmer"]["backlight"].as_str().unwrap_or("/sys/class/backlight/rpi-backlight").to_string(),
            input_devices: config["dimmer"]["input_devices"].as_vec()
                .map(|devices| devices.to_vec()).unwrap_or_default().iter()
                .map(|device| device.as_str().expect("Unsupported value for input_devices").to_string())
                .collect::<Vec<String>>(),
            dimmed_brightness: config["dimmer"]["dimmed_brightness"].as_i64().unwrap_or(30) as u8,
            full_brightness: config["dimmer"]["full_brightness"].as_i64().unwrap_or(255) as u8,
        };
        Self {
            dimmer,
        }
    }
}