use std::fs;
use std::path::Path;

pub fn get_error_response(status_code: u16, default_msg: &str) -> String {

    let error_file_path = format!("error_pages/{}.html", status_code);
    
    let body = if Path::new(&error_file_path).exists() {
        fs::read_to_string(&error_file_path).unwrap_or_else(|_| format!("<h1>{} {}</h1>", status_code, default_msg))
    } else {
        format!("<h1>{} {}</h1>", status_code, default_msg)
    };

    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code, default_msg, body.len(), body
    )
}