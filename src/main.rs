mod config;

fn main() {
    match config::Config::load("config.yaml") {
        Ok(cfg) => println!("Configuration loaded successfully: {:#?}", cfg),
        Err(e) => eprintln!("Failed to load configuration: {}", e),
    }
}