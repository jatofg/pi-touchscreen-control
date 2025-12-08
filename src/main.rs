use crate::state::State;
use log::info;
use std::sync::{Arc, RwLock};

mod config;
mod dimmer;
mod mqtt;
mod state;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        println!("Usage: pi-touchscreen-control <config file>");
        return;
    }
    let config = config::Config::new(args[1].as_str());
    let state = Arc::new(RwLock::new(State::new(
        &config.dimmer,
        dimmer::get_current_power(&config.dimmer),
        dimmer::get_current_brightness(&config.dimmer),
    )));

    let dimmer_config = config.dimmer.clone();
    let dimmer_state = state.clone();
    let dimmer_handle =
        std::thread::spawn(move || dimmer::run_dimmer(&dimmer_config, dimmer_state));

    let mqtt_config = config.mqtt.clone();
    if mqtt_config.use_mqtt {
        let mqtt_handle_state = state.clone();
        let mqtt_handle =
            std::thread::spawn(move || mqtt::run_mqtt(&mqtt_config, mqtt_handle_state));
        mqtt_handle.join().unwrap();
        info!("The MQTT thread has exited.");
    }

    dimmer_handle.join().unwrap();
    info!("The dimmer thread has exited.");
}
