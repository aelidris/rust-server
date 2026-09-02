mod cgi;
mod config;
mod error;
mod router;
mod server;

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

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::error::get_error_response;
    use crate::router::Router;

    #[test]
    fn test_config_parsing() {
        // Test that config fields parse and load correctly
        let yaml_data = "
server:
  host: \"127.0.0.1\"
  ports: [8080, 8081]
  client_max_body_size: 1048576
  error_pages:
    404: \"error_pages/404.html\"
    500: \"error_pages/500.html\"
routes:
  - path: \"/\"
    root: \"./public\"
    methods: [\"GET\"]
    default_file: \"index.html\"
    directory_listing: false
";
        let config: Config = serde_yaml::from_str(yaml_data).expect("Failed to parse config yaml");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.ports, vec![8080, 8081]);
        assert_eq!(config.server.client_max_body_size, 1048576);
    }

    #[test]
    fn test_route_matching() {
        let yaml_data = "
server:
  host: \"127.0.0.1\"
  ports: [8080]
  client_max_body_size: 1048576
  error_pages:
    404: \"error_pages/404.html\"
    500: \"error_pages/500.html\"
routes:
  - path: \"/\"
    root: \"./public\"
    methods: [\"GET\", \"DELETE\"]
    default_file: \"index.html\"
    directory_listing: false
  - path: \"/cgi-bin\"
    root: \"./cgi-bin\"
    methods: [\"GET\", \"POST\"]
    directory_listing: false
    cgi_extensions: [\".py\"]
";
        let config: Config = serde_yaml::from_str(yaml_data).unwrap();
        let router = Router::new(config);

        // Verify correct route match
        let matched = router.match_route("/");
        assert!(matched.is_some());

        let matched_cgi = router.match_route("/cgi-bin/script.py");
        assert!(matched_cgi.is_some());

        // Verify fallback route behavior for unmatched paths
        let fallback = router.match_route("/nonexistent");
        assert!(fallback.is_some());
        assert_eq!(fallback.unwrap().path, "/");
    }

    #[test]
    fn test_status_code_generation() {
        // Verify error response string formatting
        let response_404 = get_error_response(404, "Not Found");
        let response_str = String::from_utf8_lossy(response_404.as_bytes());

        assert!(response_str.contains("HTTP/1.1 404 Not Found"));
        assert!(response_str.contains("Content-Length:"));

        let response_500 = get_error_response(500, "Internal Server Error");
        let response_500_str = String::from_utf8_lossy(response_500.as_bytes());
        assert!(response_500_str.contains("HTTP/1.1 500 Internal Server Error"));
    }
}
