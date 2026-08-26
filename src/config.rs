use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub ports: Vec<u16>,
    pub client_max_body_size: usize,
    pub error_pages: HashMap<u16, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub root: String,
    pub methods: Vec<String>,
    pub default_file: Option<String>,
    pub directory_listing: bool,
    pub cgi_extensions: Option<Vec<String>>,
}

impl Config {
    // Function to load and parse the config.yaml file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
