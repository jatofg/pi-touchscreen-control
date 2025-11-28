use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread::JoinHandle;
use crate::state::State;

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
    let initial_state = State::new(&config.dimmer, dimmer::get_current_power(&config.dimmer), dimmer::get_current_brightness(&config.dimmer));

    let dimmer_config = config.dimmer.clone();
    let dimmer_state = initial_state.clone();
    let mut dimmer_handle : Option<JoinHandle<()>> = None;
    if dimmer_config.use_dimmer {
        dimmer_handle = Some(std::thread::spawn(move || dimmer::run_dimmer(&dimmer_config, &dimmer_state)));
    }

    // TODO the state must be somehow synchronized between the MQTT and the dimmer thread
    let mqtt_config = config.mqtt.clone();
    if mqtt_config.use_mqtt {
        let (state_tx, state_rx): (Sender<State>, Receiver<State>) = mpsc::channel();
        let mqtt_handle_state = initial_state.clone();
        let mqtt_handle = std::thread::spawn(move || mqtt::run_mqtt(&mqtt_config, &mqtt_handle_state, state_tx));
        let dimmer_config = config.dimmer.clone();
        let backlight_handle_state = initial_state.clone();
        let backlight_handle = std::thread::spawn(move || {
            let mut previous_state = backlight_handle_state;
            loop {
                let new_state = state_rx.recv().expect("Unable to receive power state");
                dimmer::apply_state(&dimmer_config, &previous_state, &new_state);
                previous_state = new_state;
            }
        });
        mqtt_handle.join().unwrap();
        backlight_handle.join().unwrap();
    }

    if let Some(dimmer_handle) = dimmer_handle {
        dimmer_handle.join().unwrap();
    }
}
