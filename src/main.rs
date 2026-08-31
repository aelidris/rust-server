mod config;
mod server;
mod router;
mod cgi;
mod error;

pub mod utils {
    pub mod cookie;
    pub mod session;
}

use server::Server;

fn main() {
    match config::Config::load("config.yaml") {
        Ok(cfg) => {
            println!("Configuration loaded successfully!");
            let mut server = Server::new(cfg);
            if let Err(e) = server.run() {
                eprintln!("Server error: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to load configuration: {}", e),
    }
}