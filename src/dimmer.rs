use crate::config::DimmerConfig;
use crate::state::State;
use log::info;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

fn set_backlight_power(config: &DimmerConfig, power: bool) {
    let power_path = config.backlight.clone() + "/bl_power";
    let mut power_file = OpenOptions::new()
        .read(false)
        .write(true)
        .open(&power_path)
        .expect("Unable to open bl_power file");
    let power_buf: &[u8; 1] = if power { b"0" } else { b"1" };
    power_file
        .write_all(power_buf)
        .expect("Unable to write bl_power");
    info!("Backlight power set to {}", power);
}

fn set_brightness(config: &DimmerConfig, brightness: u8) {
    let brightness_path = config.backlight.clone() + "/brightness";
    let mut brightness_file = OpenOptions::new()
        .read(false)
        .write(true)
        .open(&brightness_path)
        .expect("Unable to open brightness file");
    let brightness_buf = brightness.to_string().into_bytes();
    brightness_file
        .write_all(brightness_buf.as_slice())
        .expect("Unable to write brightness");
    info!("Brightness set to {}", brightness);
}

fn dim_down(current_state: &State, state: &Arc<RwLock<State>>) {
    if (current_state.backlight_active_current && !current_state.backlight_active_dimmed)
        || current_state.brightness_current != current_state.brightness_dimmed
    {
        info!("Dimming down due to inactivity");
        let mut state = state
            .write()
            .expect("Unable to acquire write lock on state");
        state.backlight_active_current = state.backlight_active_dimmed;
        state.brightness_current = state.brightness_dimmed;
    }
}

fn dim_up(current_state: &State, state: &Arc<RwLock<State>>) {
    if !current_state.backlight_active_current
        || current_state.brightness_current != current_state.brightness_full
    {
        info!("Dimming up due to touch input");
        let mut state = state
            .write()
            .expect("Unable to acquire write lock on state");
        state.backlight_active_current = true;
        state.brightness_current = state.brightness_full;
    }
}

fn apply_state(config: &DimmerConfig, previous_state: &State, new_state: &State) {
    if previous_state.backlight_active_current != new_state.backlight_active_current {
        set_backlight_power(config, new_state.backlight_active_current);
    }
    if previous_state.brightness_current != new_state.brightness_current {
        set_brightness(config, new_state.brightness_current);
    }
}

pub fn get_current_brightness(config: &DimmerConfig) -> u8 {
    let path = config.backlight.clone() + "/actual_brightness";
    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .open(path)
        .expect("Unable to open actual_brightness");
    let mut brightness_str = String::new();
    file.read_to_string(&mut brightness_str)
        .expect("Unable to read from actual_brightness");
    brightness_str
        .trim()
        .parse::<u8>()
        .expect(("Unable to parse brightness: ".to_string() + brightness_str.as_str()).as_str())
}

pub fn get_current_power(config: &DimmerConfig) -> bool {
    let path = config.backlight.clone() + "/bl_power";
    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .open(path)
        .expect("Unable to open bl_power");
    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer)
        .expect("Unable to read from bl_power");
    buffer == *b"0"
}

pub fn run_dimmer(config: &DimmerConfig, state: Arc<RwLock<State>>) {
    let mut devices = Vec::new();
    for device_path in config.input_devices.iter() {
        devices.push(
            OpenOptions::new()
                .read(true)
                .write(false)
                .custom_flags(libc::O_NONBLOCK)
                .open(&device_path)
                .expect(format!("Unable to open device {device_path}").as_str()),
        );
    }
    let mut last_touch = Instant::now();
    let mut previous_state = state
        .read()
        .expect("Unable to acquire read lock on state")
        .clone();

    loop {
        let current_state = state
            .read()
            .expect("Unable to acquire read lock on state")
            .clone();
        if current_state != previous_state {
            apply_state(config, &previous_state, &current_state);
        }

        if current_state.use_dimmer {
            for device in devices.iter_mut() {
                let mut buffer = [0u8, 8];
                let mut new_touch = false;
                while device.read(&mut buffer).is_ok() {
                    new_touch = true;
                }
                if new_touch {
                    last_touch = Instant::now();
                    dim_up(&current_state, &state);
                }
            }

            if last_touch.elapsed() > current_state.timeout {
                dim_down(&current_state, &state);
            }
        }

        previous_state = current_state;
        std::thread::sleep(Duration::from_millis(100));
    }
}
