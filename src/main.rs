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
    let dimmer_handle = std::thread::spawn(move || dimmer::run_dimmer(&dimmer_config));
    let mqtt_config = config.mqtt.clone();
    // TODO check use_mqtt
    let mqtt_handle = std::thread::spawn(move || mqtt::run_mqtt(&mqtt_config));
    dimmer_handle.join().unwrap();
    mqtt_handle.join().unwrap();
}
