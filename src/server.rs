use crate::config::Config;
use crate::error::get_error_response;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::collections::HashMap;

const SERVER_TOKEN: Token = Token(0);

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
        let port = self.config.server.ports[0];
        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

        let mut listener = TcpListener::bind(addr)?;
        println!("Server listening on http://{}", addr);

        poll.registry()
            .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

        // Map to keep track of connected client streams by their unique Token
        let mut clients: HashMap<usize, TcpStream> = HashMap::new();
        let mut next_token_id = 1;

        loop {
            poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        // Accept incoming connections
                        loop {
                            match listener.accept() {
                                Ok((mut stream, remote_addr)) => {
                                    println!("Accepted new connection from: {}", remote_addr);
                                    
                                    let client_token = Token(next_token_id);
                                    next_token_id += 1;

                                    // Register the client stream with Mio for readability
                                    poll.registry().register(
                                        &mut stream,
                                        client_token,
                                        Interest::READABLE,
                                    )?;

                                    clients.insert(client_token.0, stream);
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    break; // No more connections ready
                                }
                                Err(e) => {
                                    eprintln!("Error accepting connection: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Token(id) => {
                        // Handle data from an existing client connection
                        if let Some(stream) = clients.get_mut(&id) {
                            let mut buf = [0; 1024];
                            match stream.read(&mut buf) {
                                Ok(0) => {
                                    println!("Client disconnected (token {})", id);
                                    clients.remove(&id);
                                }
                                Ok(n) => {
                                    let request_str = String::from_utf8_lossy(&buf[..n]);
                                    println!("Received request data from client {}:\n{}", id, request_str);

                                    // INSERT ROUTER LOGIC
                                    let mut lines = request_str.lines();
                                    if let Some(request_line) = lines.next() {
                                        let parts: Vec<&str> = request_line.split_whitespace().collect();
                                        if parts.len() >= 2 {
                                            let method = parts[0];
                                            let path = parts[1];

                                            // Collect headers first so we can check sessions/cookies                                         
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

                                            // Check session routes (/login, /profile) first using persistent self.router
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

                                                    // CHECK IF IT'S A CGI REQUEST
                                                    if self.router.is_cgi_request(route, &file_path) {
                                                        println!("Triggering CGI execution for: {:?}", file_path);
                                                        
                                                        // Extract query string if present (e.g., /cgi-bin/test.py?name=test)
                                                        let path_parts: Vec<&str> = path.splitn(2, '?').collect();
                                                        let query_string = path_parts.get(1).copied();

                                                        // Collect headers and find content length
                                                        let mut headers = HashMap::new();
                                                        let mut content_length = 0;
                                                        let mut body_bytes: Vec<u8> = Vec::new();
                                                        
                                                        let mut lines_iter = lines.peekable();
                                                        for line in &mut lines_iter {
                                                            if line.is_empty() { break; }
                                                            if let Some((key, val)) = line.split_once(':') {
                                                                let k = key.trim().to_lowercase();
                                                                let v = val.trim().to_string();
                                                                if k == "content-length" {
                                                                    content_length = v.parse().unwrap_or(0);
                                                                }
                                                                headers.insert(k, v);
                                                            }
                                                        }

                                                        // If it's a POST request with content, read the body
                                                        if method.eq_ignore_ascii_case("POST") && content_length > 0 {
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

                                                        let body_arg = if body_bytes.is_empty() { None } else { Some(body_bytes.as_slice()) };

                                                        // Execute the CGI script with the body payload
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
                                                        // Fallback: Regular Static File Serving or DELETE handling
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
                                                            // Regular GET file serving code
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
                                    // ---> END OF ROUTER LOGIC <---

                                    clients.remove(&id);
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    // The socket isn't ready with more data right now, so just do nothing and let the main loop continue polling other events without crashing
                                }
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
}