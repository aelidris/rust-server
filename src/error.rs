use std::fs;
use std::path::Path;

pub fn get_error_response(status_code: u16, default_message: &str) -> String {
    let file_path = format!("error_pages/{}.html", status_code);
    
    let body = if Path::new(&file_path).exists() {
        fs::read_to_string(&file_path).unwrap_or_else(|_| format!("<html><body><h1>{} - {}</h1></body></html>", status_code, default_message))
    } else {
        format!("<html><body><h1>{} - {}</h1></body></html>", status_code, default_message)
    };

    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        status_code, default_message, body.len(), body
    )
}