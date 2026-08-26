use crate::config::Config;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::collections::HashMap;

const SERVER_TOKEN: Token = Token(0);

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self { config }
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

                                            let router = crate::router::Router::new(self.config.clone());

                                            if let Some(route) = router.match_route(path) {
                                                if router.is_method_allowed(route, method) {
                                                    let file_path = router.resolve_file_path(route, path);
                                                    println!("Matched route! Resolved file path: {:?}", file_path);

                                                    // Read file from filesystem instead of hardcoded string
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
                                                            let body = "<h1>404 Not Found</h1>";
                                                            let response = format!(
                                                                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n{}",
                                                                body.len(),
                                                                body
                                                            );
                                                            let _ = stream.write_all(response.as_bytes());
                                                        }
                                                    }
                                                } else {
                                                    let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                                                    let _ = stream.write_all(response.as_bytes());
                                                }
                                            } else {
                                                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                                                let _ = stream.write_all(response.as_bytes());
                                            }
                                        }
                                    }
                                    // ---> END OF ROUTER LOGIC <---

                                    clients.remove(&id);
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    // Not ready yet
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