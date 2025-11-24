use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

mod config;
mod dimmer;
mod mqtt;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        println!("Usage: pi-touchscreen-control <config file>");
        return;
    }
    let config = config::Config::new(args[1].as_str());

    let dimmer_config = config.dimmer.clone();
    dimmer::set_backlight_power(&dimmer_config, true);
    let dimmer_handle = std::thread::spawn(move || dimmer::run_dimmer(&dimmer_config));

    let mqtt_config = config.mqtt.clone();

    if mqtt_config.use_mqtt {
        let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
        let mqtt_handle = std::thread::spawn(move || mqtt::run_mqtt(&mqtt_config, tx));
        let dimmer_config = config.dimmer.clone();
        let backlight_handle = std::thread::spawn(move || {
            loop {
                let power = rx.recv().expect("Unable to receive power state");
                dimmer::set_backlight_power(&dimmer_config, power);
            }
        });
        mqtt_handle.join().unwrap();
        backlight_handle.join().unwrap();
    }

    dimmer_handle.join().unwrap();

}
