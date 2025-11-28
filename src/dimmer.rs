use crate::config::DimmerConfig;
use crate::state::State;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::io::{Read, Write};
use std::time::Instant;

fn set_backlight_power(config: &DimmerConfig, power: bool) {
    let power_path = config.backlight.clone() + "/bl_power";
    let mut power_file = OpenOptions::new().read(false).write(true).open(&power_path)
        .expect("Unable to open bl_power file");
    let power_buf: &[u8; 1] = if power { b"0" } else { b"1" };
    power_file.write_all(power_buf).expect("Unable to write bl_power");
}

fn set_brightness(config: &DimmerConfig, brightness: u8) {
    let brightness_path = config.backlight.clone() + "/brightness";
    let mut brightness_file = OpenOptions::new().read(false).write(true).open(&brightness_path)
        .expect("Unable to open brightness file");
    let brightness_buf = brightness.to_string().into_bytes();
    brightness_file.write_all(brightness_buf.as_slice()).expect("Unable to write brightness");
}

fn dim_down(config: &DimmerConfig, state: &mut State) {
    if config.turn_backlight_off && state.backlight_active_current {
        set_backlight_power(config, false);
        state.backlight_active_current = false;
    }
    if !config.turn_backlight_off && state.brightness_current > config.dimmed_brightness {
        set_brightness(config, config.dimmed_brightness);
        state.brightness_current = config.dimmed_brightness;
    }
}

fn dim_up(config: &DimmerConfig, state: &mut State) {
    if !state.backlight_active_current {
        set_backlight_power(config, true);
        state.backlight_active_current = true;
    }
    if state.brightness_current < config.full_brightness {
        set_brightness(config, config.full_brightness);
        state.brightness_current = config.full_brightness;
    }
}

pub fn get_current_brightness(config: &DimmerConfig) -> u8 {
    let path = config.backlight.clone() + "/actual_brightness";
    let mut file = OpenOptions::new().read(true).write(false).open(path).expect("Unable to open actual_brightness");
    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer).expect("Unable to read from actual_brightness");
    buffer[0]
}

pub fn get_current_power(config: &DimmerConfig) -> bool {
    let path = config.backlight.clone() + "/bl_power";
    let mut file = OpenOptions::new().read(true).write(false).open(path).expect("Unable to open bl_power");
    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer).expect("Unable to read from bl_power");
    buffer[0] == 0
}

pub fn apply_state(config: &DimmerConfig, previous_state: &State, new_state: &State) {
    if previous_state.use_dimmer != new_state.use_dimmer {
        set_backlight_power(config, new_state.use_dimmer);
    }
    if previous_state.brightness_current != new_state.brightness_current {
        set_brightness(config, new_state.brightness_current);
    }
}

pub fn run_dimmer(config: &DimmerConfig, initial_state: &State) {
    let mut devices = Vec::new();
    for device_path in config.input_devices.iter() {
        devices.push(OpenOptions::new().read(true).write(false).custom_flags(libc::O_NONBLOCK).open(&device_path)
            .expect(format!("Unable to open device {device_path}").as_str()));
    }
    // TODO move
    //let mut current_brightness = get_current_brightness((config.backlight.clone() + "/actual_brightness").as_ref());
    let mut state = initial_state.clone();
    let mut last_touch = Instant::now();

    loop {
        for device in devices.iter_mut() {
            let mut buffer = [0u8, 8];
            let mut new_touch = false;
            while device.read(&mut buffer).is_ok() {
                new_touch = true;
            }
            if new_touch {
                last_touch = Instant::now();
                dim_up(config, &mut state);
            }
        }

        if last_touch.elapsed() > config.timeout {
            dim_down(config, &mut state);
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}