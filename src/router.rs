use crate::config::{Config, RouteConfig};
use std::path::PathBuf;

pub struct Router {
    config: Config,
}

impl Router {
    pub fn new(config: Config) -> Self {
        Self { config }
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
        route.methods.iter().any(|m| m.eq_ignore_ascii_case(method))
    }

    /// Resolves the file system path for a matched static route
    pub fn resolve_file_path(&self, route: &RouteConfig, request_path: &str) -> PathBuf {
        let trimmed_path = request_path.strip_prefix(&route.path).unwrap_or(request_path);
        let mut full_path = PathBuf::from(&route.root);
        
        if trimmed_path.is_empty() || trimmed_path == "/" {
            if let Some(ref default_file) = route.default_file {
                full_path.push(default_file);
            }
        } else {
            full_path.push(trimmed_path.strip_prefix("/").unwrap_or(trimmed_path));
        }

        full_path
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
}