use crate::config::Config;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

fn read_actual_brightness(path: &Path) -> u8 {
    let mut file = OpenOptions::new().read(true).write(false).open(path).expect("Unable to open actual_brightness");
    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer).expect("Unable to read from actual_brightness");
    buffer[0]
}

pub fn run_dimmer(config: &Config) {
    let mut devices = Vec::new();
    for device_path in config.dimmer.input_devices.iter() {
        devices.push(OpenOptions::new().read(true).write(false).open(&device_path)
            .expect(format!("Unable to open device {device_path}").as_str()));
    }
    let mut current_brightness = read_actual_brightness((config.dimmer.backlight.clone() + "/actual_brightness").as_ref());
    let full_brightness = config.dimmer.full_brightness;
    let dimmed_brightness = config.dimmer.dimmed_brightness;

    let brightness_path = config.dimmer.backlight.clone() + "/brightness";
    let mut brightness_file = OpenOptions::new().read(false).write(true).open(&brightness_path)
        .expect("Unable to open brightness file");
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
                brightness_file.write_all(&[full_brightness]).expect("Unable to write brightness");
                current_brightness = full_brightness;
            }
        }

        if last_touch.elapsed() > config.dimmer.timeout {
            if current_brightness != dimmed_brightness {
                brightness_file.write_all(&[dimmed_brightness]).expect("Unable to write brightness");
                current_brightness = dimmed_brightness;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}