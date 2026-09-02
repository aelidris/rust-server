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
curl -i http://127.0.0.1:8080/index.html
```

### Test DELETE:

```bash
curl -X DELETE -i http://127.0.0.1:8080/delete_me.txt
```

### Test CGI Script (POST):

``` bash
curl -X POST -d "param=value" -i http://127.0.0.1:8080/cgi-bin/post_test.py
```

### Test Custom Error Pages (404 / 405):

#### Test 404 Not Found
``` bash
curl -i http://127.0.0.1:8080/non_existent_page.html
```
#### Test 405 Method Not Allowed
``` bash
curl -X POST -i http://127.0.0.1:8080/index.html
```

### State Management & Cookies
The server includes a robust in-memory session manager and cookie parser (`utils/cookie.rs` and `utils/session.rs`) built to handle persistent user authentication states across non-blocking I/O connections:
* **`/login`**: Generates a cryptographically secure session ID, registers it within the persistent session store, and issues an `HttpOnly` `Set-Cookie` header.
* **`/profile`**: Parses incoming `Cookie` headers, extracts the `session_id`, and validates it against active server sessions to authorize access.


### Testing State Management
You can verify the session and cookie functionality using `curl`:

1. **Test Login (Generates Session & Cookie):**
``` bash
curl -i http://127.0.0.1:8080/login
```

2. **Test Profile (Validates Session Cookie):**
``` bash
curl -i http://127.0.0.1:8080/profile -H "Cookie: session_id=YOUR_GENERATED_ID"
```

### Connection Management & Chunked Requests

***Idle Connection Timeout: Automatically sweeps and cleans up inactive client sockets after 5 seconds using mio polling intervals and an Instant-based tracking sweep.***

***Chunked Transfer Encoding: Natively parses stream-based POST requests using hex-length size decoding and 0\r\n\r\n boundary termination.***

#### Testing Chunked Transfer Encoding
You can verify the chunked request processing using Python or a raw socket script:
``` bash
python3 -c '
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect(("127.0.0.1", 8080))
s.sendall(b"POST /cgi-bin/test.py HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n")
s.sendall(b"5\r\nHello\r\n")
s.sendall(b"7\r\n World!\r\n")
s.sendall(b"0\r\n\r\n")
print(s.recv(4096).decode(errors="ignore"))
s.close()
'
```

###Error Handling & Payload Limits

***Dynamic Error Pages: Automatically reads and serves custom HTML error templates from the error_pages/ directory for codes 400, 403, 404, 405, 413, and 500.***

***Payload Enforcement: Validates incoming Content-Length headers against the YAML-configured client_max_body_size, returning a 413 Payload Too Large error response if exceeded.***

#### Testing Error Pages & Payload Limits

#### Test 413 Payload Too Large

``` bash
curl -X POST -i http://127.0.0.1:8080/cgi-bin/test.py \
  -H "Content-Length: 2000000" \
  -d "exceeding limit body data"
```

### Test 500 Internal Server Error
``` bash
curl -i -X POST http://127.0.0.1:8080/cgi-bin/error_test.py
```