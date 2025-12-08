use std::time::Duration;
use crate::config;

#[derive(Clone, PartialEq, Debug)]
pub struct State {
    pub backlight_active_current: bool,
    pub backlight_active_dimmed: bool,
    pub brightness_current: u8,
    pub brightness_dimmed: u8,
    pub brightness_full: u8,
    pub use_dimmer: bool,
    pub timeout: Duration,
}

impl State {
    pub fn new(config: &config::DimmerConfig, backlight_active_current: bool, brightness_current: u8) -> Self {
        Self {
            backlight_active_current,
            backlight_active_dimmed: !config.turn_backlight_off,
            brightness_current,
            brightness_dimmed: config.dimmed_brightness,
            brightness_full: config.full_brightness,
            use_dimmer: config.use_dimmer,
            timeout: config.timeout,
        }
    }
}