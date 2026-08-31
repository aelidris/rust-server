use std::collections::HashMap;

/// Parses a "Cookie" header string into a key-value map.
/// Example header: "session_id=abc123xyz; user=Alice"
pub fn parse_cookies(cookie_header: &str) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for cookie in cookie_header.split(';') {
        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
        if parts.len() == 2 {
            cookies.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    cookies
}

/// Formats a "Set-Cookie" header string.
pub fn create_set_cookie_header(name: &str, value: &str, max_age: Option<u64>) -> String {
    let mut header = format!("Set-Cookie: {}={}; Path=/; HttpOnly", name, value);
    if let Some(age) = max_age {
        header.push_str(&format!("; Max-Age={}", age));
    }
    header
}