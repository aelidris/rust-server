# LocalServer (Rust)

A custom HTTP/1.1-compliant web server built from scratch in Rust, utilizing non-blocking I/O (`mio`) and running on a single-threaded event loop.

## Features
- Non-blocking event-driven architecture
- Support for `GET`, `POST`, and `DELETE` methods
- File uploads, chunked requests, cookies, and sessions
- CGI script execution via `std::process::Command`
- Flexible configuration parsing (routes, hosts, ports, error pages via YAML)
- Crash-proof resilience under heavy load testing (`siege`)

## Prerequisites

Make sure you have Rust and Cargo installed on your system. This project requires:
- **Rust / Cargo** version `1.98.0` or higher (fully compatible with Cargo editions `2021+`)

You can verify your installation by running:
```bash
rustc --version
cargo --version