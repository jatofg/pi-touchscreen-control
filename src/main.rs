mod config;
mod dimmer;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        println!("Usage: pi-touchscreen-control <config file>");
        return;
    }
    let config = config::Config::new(args[1].as_str());
    dimmer::run_dimmer(&config);
}
