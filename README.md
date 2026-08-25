# LocalServer (Rust)

A custom HTTP/1.1-compliant web server built from scratch in Rust, utilizing non-blocking I/O (`mio`) and running on a single-threaded event loop.

## Features
- Non-blocking event-driven architecture
- Support for `GET`, `POST`, and `DELETE` methods
- File uploads, chunked requests, cookies, and sessions
- CGI script execution via `std::process::Command`
- Flexible configuration parsing (routes, hosts, ports, error pages)
- Crash-proof resilience under heavy load testing (`siege`)