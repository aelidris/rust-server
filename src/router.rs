use crate::config::{Config, RouteConfig};
use crate::utils::cookie::{parse_cookies, create_set_cookie_header};
use crate::utils::session::SessionManager;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct Router {
    config: Config,
    pub session_manager: SessionManager,
}

impl Router {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            session_manager: SessionManager::new(3600), // 1 hour session duration
        }
    }

    /// Finds a matching route for a given path, or returns None if 404
    pub fn match_route(&self, path: &str) -> Option<&RouteConfig> {
        // Find the longest matching path prefix or exact match
        self.config
            .routes
            .iter()
            .filter(|route| path.starts_with(&route.path))
            .max_by_key(|route| route.path.len())
    }

    /// Checks if a given HTTP method is allowed for a specific route
    pub fn is_method_allowed(&self, route: &RouteConfig, method: &str) -> bool {
        if let Some(methods) = &route.methods {
            methods.iter().any(|m| m.eq_ignore_ascii_case(method))
        } else {
            false
        }
    }

    /// Resolves the file system path for a matched static route
    pub fn resolve_file_path(&self, route: &RouteConfig, request_path: &str) -> PathBuf {
        let root = route.root.as_deref().unwrap_or("./public");
        let trimmed_path = request_path.strip_prefix(&route.path).unwrap_or(request_path);
        let mut full_path = PathBuf::from(root);
        
        if trimmed_path.is_empty() || trimmed_path == "/" {
            if let Some(ref default_file) = route.default_file {
                full_path.push(default_file);
            }
        } else {
            full_path.push(trimmed_path.strip_prefix("/").unwrap_or(trimmed_path));
        }

        full_path
    }

    /// Checks if a matched route defines a redirection path
    pub fn get_redirection<'a>(&self, route: &'a RouteConfig) -> Option<&'a String> {
        route.redirect.as_ref()
    }

    /// Checks if a request target and matched route qualify for CGI execution
    pub fn is_cgi_request(&self, route: &RouteConfig, file_path: &PathBuf) -> bool {
        if let Some(extensions) = &route.cgi_extensions {
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                return extensions.iter().any(|allowed| allowed.trim_start_matches('.') == ext);
            }
        }
        false
    }

    /// Handles session-based test routes (/login && /profile)
    pub fn handle_session_route(&mut self, path: &str, headers: &HashMap<String, String>) -> Option<(String, String)> {
        if path == "/login" {
            let session_id = self.session_manager.create_session();
            let cookie_header = create_set_cookie_header("session_id", &session_id, Some(3600));
            let body = "<html><body><h1>Session Created & Cookie Set Successfully!</h1></body></html>";
            return Some((cookie_header, body.to_string()));
        }

        if path == "/profile" {
            if let Some(cookie_str) = headers.get("cookie").or_else(|| headers.get("Cookie")) {
                let cookies = parse_cookies(cookie_str);
                if let Some(session_id) = cookies.get("session_id") {
                    if self.session_manager.get_session(session_id).is_some() {
                        let body = "<html><body><h1>Profile Page: Valid Session Found!</h1></body></html>";
                        return Some(("".to_string(), body.to_string()));
                    }
                }
            }
            let body = "<html><body><h1>Unauthorized: No valid session cookie found.</h1></body></html>";
            return Some(("".to_string(), body.to_string()));
        }

        None
    }
}