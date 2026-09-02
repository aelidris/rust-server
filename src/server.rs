use crate::config::Config;
use crate::error::get_error_response;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::{Instant, Duration};

struct ClientConnection {
    stream: TcpStream,
    last_active: Instant,
}

pub struct Server {
    config: Config,
    router: crate::router::Router,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let router = crate::router::Router::new(config.clone());
        Self { config, router }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(1024);

        let host = &self.config.server.host;
        let mut listeners: HashMap<usize, TcpListener> = HashMap::new();

        // Bind and register a listener for every port in the config
        for (index, &port) in self.config.server.ports.iter().enumerate() {
            let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
            let mut listener = TcpListener::bind(addr)?;
            println!("Server listening on http://{}", addr);

            let listener_token = Token(index);
            poll.registry().register(
                &mut listener,
                listener_token,
                Interest::READABLE,
            )?;

            listeners.insert(index, listener);
        }

        // Map to keep track of connected client streams and timestamps by Token
        let mut clients: HashMap<usize, ClientConnection> = HashMap::new();
        let mut next_token_id = self.config.server.ports.len();
        let timeout_duration = Duration::from_secs(5); // 5 seconds request timeout

        loop {
            // Poll with a 500ms timeout so the event loop wakes up periodically
            poll.poll(&mut events, Some(Duration::from_millis(500)))?;

            // Sweep and clean up expired connections
            let now = Instant::now();
            let mut timed_out_ids = Vec::new();
            for (&id, client) in &clients {
                if now.duration_since(client.last_active) > timeout_duration {
                    timed_out_ids.push(id);
                }
            }
            for id in timed_out_ids {
                println!("Connection timeout for client token {}", id);
                if let Some(mut client) = clients.remove(&id) {
                    let _ = poll.registry().deregister(&mut client.stream);
                }
            }

            for event in events.iter() {
                let token_id = event.token().0;

                // Check if the event belongs to one of our multi-port listeners
                if let Some(listener) = listeners.get_mut(&token_id) {
                    loop {
                        match listener.accept() {
                            Ok((mut stream, remote_addr)) => {
                                println!("Accepted new connection from: {}", remote_addr);
                                
                                let client_token = Token(next_token_id);
                                next_token_id += 1;

                                poll.registry().register(
                                    &mut stream,
                                    client_token,
                                    Interest::READABLE,
                                )?;

                                clients.insert(client_token.0, ClientConnection {
                                    stream,
                                    last_active: Instant::now(),
                                });
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error accepting connection: {}", e);
                                break;
                            }
                        }
                    }
                } else {
                    // Handle existing client data streams
                    let id = token_id;
                    if let Some(client) = clients.get_mut(&id) {
                        client.last_active = Instant::now(); // Refresh timeout timer on activity
                        let stream = &mut client.stream;

                        let mut buf = [0; 1024];
                        match stream.read(&mut buf) {
                            Ok(0) => {
                                println!("Client disconnected (token {})", id);
                                clients.remove(&id);
                            }
                            Ok(n) => {
                                let request_str = String::from_utf8_lossy(&buf[..n]);
                                println!("Received request data from client {}:\n{}", id, request_str);

                                let mut lines = request_str.lines();
                                if let Some(request_line) = lines.next() {
                                    let parts: Vec<&str> = request_line.split_whitespace().collect();
                                    if parts.len() >= 2 {
                                        let method = parts[0];
                                        let path = parts[1];                                         
                                        let mut headers = HashMap::new();
                                        
                                        let mut lines_iter = lines.clone().peekable();
                                        for line in &mut lines_iter {
                                            if line.is_empty() { break; }
                                            if let Some((key, val)) = line.split_once(':') {
                                                let k = key.trim().to_lowercase();
                                                let v = val.trim().to_string();
                                                headers.insert(k, v);
                                            }
                                        }

                                        if let Some((cookie_header, body)) = self.router.handle_session_route(path, &headers) {
                                            let response = if !cookie_header.is_empty() {
                                                format!(
                                                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n{}\r\nContent-Length: {}\r\n\r\n{}",
                                                    cookie_header,
                                                    body.len(),
                                                    body
                                                )
                                            } else {
                                                format!(
                                                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                                                    body.len(),
                                                    body
                                                )
                                            };
                                            let _ = stream.write_all(response.as_bytes());
                                            clients.remove(&id);
                                            continue;
                                        }

                                        if let Some(route) = self.router.match_route(path) {
                                            if self.router.is_method_allowed(route, method) {
                                                let file_path = self.router.resolve_file_path(route, path);
                                                println!("Matched route! Resolved path: {:?}", file_path);

                                                if self.router.is_cgi_request(route, &file_path) {
                                                    println!("Triggering CGI execution for: {:?}", file_path);
                                                    
                                                    let path_parts: Vec<&str> = path.splitn(2, '?').collect();
                                                    let query_string = path_parts.get(1).copied();

                                                    let mut headers = HashMap::new();
                                                    let mut content_length = 0;
                                                    let mut body_bytes: Vec<u8> = Vec::new();
                                                    let mut is_chunked = false;
                                                    
                                                    let mut lines_iter = lines.peekable();
                                                    for line in &mut lines_iter {
                                                        if line.is_empty() { break; }
                                                        if let Some((key, val)) = line.split_once(':') {
                                                            let k = key.trim().to_lowercase();
                                                            let v = val.trim().to_string();
                                                            if k == "content-length" {
                                                                content_length = v.parse().unwrap_or(0);
                                                            } else if k == "transfer-encoding" && v.to_lowercase().contains("chunked") {
                                                                is_chunked = true;
                                                            }
                                                            headers.insert(k, v);
                                                        }
                                                    }

                                                    if method.eq_ignore_ascii_case("POST") {
                                                        if is_chunked {
                                                            // Find where headers end (\r\n\r\n) in the initial read buffer
                                                            let remaining_bytes = if let Some(pos) = request_str.find("\r\n\r\n") {
                                                                let body_start_offset = pos + 4;
                                                                // Get the slice of bytes from our original buffer starting after headers
                                                                &buf[body_start_offset..n]
                                                            } else {
                                                                &[]
                                                            };
                                                            body_bytes = read_chunked_body(stream, remaining_bytes)?;
                                                        } else if content_length > 0 {
                                                            let remaining_text: String = lines_iter.collect::<Vec<&str>>().join("\n");
                                                            let mut body = remaining_text.into_bytes();
                                                            
                                                            while body.len() < content_length {
                                                                let mut chunk = vec![0; content_length - body.len()];
                                                                match stream.read(&mut chunk) {
                                                                    Ok(0) => break,
                                                                    Ok(n) => body.extend_from_slice(&chunk[..n]),
                                                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                                                        continue;
                                                                    }
                                                                    Err(_) => break,
                                                                }
                                                            }
                                                            body_bytes = body;
                                                        }
                                                    }

                                                    let body_arg = if body_bytes.is_empty() { None } else { Some(body_bytes.as_slice()) };

                                                    match crate::cgi::execute_cgi(&file_path, method, query_string, body_arg, &headers) {
                                                        Ok(script_output) => {
                                                            let response = format!(
                                                                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                                                                script_output.len()
                                                            );
                                                            let _ = stream.write_all(response.as_bytes());
                                                            let _ = stream.write_all(&script_output);
                                                        }
                                                        Err(e) => {
                                                            eprintln!("CGI Execution Error: {}", e);
                                                            let response = get_error_response(500, "Internal Server Error");
                                                            let _ = stream.write_all(response.as_bytes());
                                                        }
                                                    }
                                                } else {
                                                    if method.eq_ignore_ascii_case("DELETE") {
                                                        if file_path.is_file() {
                                                            match std::fs::remove_file(&file_path) {
                                                                Ok(_) => {
                                                                    let response = format!(
                                                                        "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n"
                                                                    );
                                                                    let _ = stream.write_all(response.as_bytes());
                                                                }
                                                                Err(_) => {
                                                                    let response = get_error_response(403, "Forbidden");
                                                                    let _ = stream.write_all(response.as_bytes());
                                                                }
                                                            }
                                                        } else {
                                                            let response = get_error_response(404, "Not Found");
                                                            let _ = stream.write_all(response.as_bytes());
                                                        }
                                                    } else {
                                                        match std::fs::read_to_string(&file_path) {
                                                            Ok(contents) => {
                                                                let response = format!(
                                                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                                                                    contents.len(),
                                                                    contents
                                                                );
                                                                let _ = stream.write_all(response.as_bytes());
                                                            }
                                                            Err(_) => {
                                                                let response = get_error_response(404, "Not Found");
                                                                let _ = stream.write_all(response.as_bytes());
                                                            }
                                                        }
                                                    }                                                 
                                                }
                                            } else {
                                                let response = get_error_response(405, "Method Not Allowed");
                                                let _ = stream.write_all(response.as_bytes());
                                            }
                                        } else {
                                            let response = get_error_response(404, "Not Found");
                                            let _ = stream.write_all(response.as_bytes());
                                        }
                                    }
                                }
                                clients.remove(&id);
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                            Err(e) => {
                                eprintln!("Error reading from client {}: {}", id, e);
                                clients.remove(&id);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn read_chunked_body<R: Read>(stream: &mut R, initial_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut body = Vec::new();
    // Create a chained reader so we process initial buffered bytes before reading from the stream
    let mut buffered_stream = std::io::Cursor::new(initial_data).chain(stream);
    let mut line_buf = Vec::new();

    loop {
        line_buf.clear();
        loop {
            let mut byte = [0; 1];
            match buffered_stream.read(&mut byte) {
                Ok(0) => break,
                Ok(_) => {
                    line_buf.push(byte[0]);
                    if line_buf.ends_with(b"\r\n") {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(e),
            }
        }

        let line_str = String::from_utf8_lossy(&line_buf);
        let hex_str = line_str.trim();
        
        let chunk_size = usize::from_str_radix(hex_str, 16).unwrap_or(0);
        if chunk_size == 0 {
            let mut terminator = [0; 2];
            let _ = buffered_stream.read(&mut terminator);
            break;
        }

        let mut chunk_data = vec![0; chunk_size];
        buffered_stream.read_exact(&mut chunk_data)?;
        body.extend_from_slice(&chunk_data);

        let mut crlf = [0; 2];
        let _ = buffered_stream.read(&mut crlf);
    }

    Ok(body)
}