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
- **Rust / Cargo** version `1.98.0` or higher

You can verify your installation by running:
```bash
rustc --version
cargo --version
```

## Running the Server
### Start the server using Cargo:

``` bash
cargo run
```

## Testing Endpoints

### Test GET (Static File):

``` bash
curl -i [http://127.0.0.1:8080/index.html](http://127.0.0.1:8080/index.html)
```

### Test DELETE:

```bash
curl -X DELETE -i [http://127.0.0.1:8080/delete_me.txt](http://127.0.0.1:8080/delete_me.txt)
```

### Test CGI Script (POST):

``` bash
curl -X POST -d "param=value" -i [http://127.0.0.1:8080/cgi-bin/post_test.py](http://127.0.0.1:8080/cgi-bin/post_test.py)
```