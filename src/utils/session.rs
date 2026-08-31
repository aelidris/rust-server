use std::collections::HashMap;
use std::time::{Instant, Duration};

#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub data: HashMap<String, String>,
    pub expires_at: Instant,
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    session_duration: Duration,
}

impl SessionManager {
    pub fn new(duration_secs: u64) -> Self {
        Self {
            sessions: HashMap::new(),
            session_duration: Duration::from_secs(duration_secs),
        }
    }

    /// Generates a new random session ID and registers a session.
    pub fn create_session(&mut self) -> String {
        // Clean up expired sessions first
        self.cleanup_expired();

        let id = format!("{:x}", rand_id());
        let session = Session {
            id: id.clone(),
            data: HashMap::new(),
            expires_at: Instant::now() + self.session_duration,
        };

        self.sessions.insert(id.clone(), session);
        id
    }

    /// Retrieves a mutable reference to an active session by ID.
    pub fn get_session(&mut self, id: &str) -> Option<&mut Session> {
        self.cleanup_expired();
        if let Some(session) = self.sessions.get_mut(id) {
            if session.expires_at > Instant::now() {
                // Refresh expiration on activity
                session.expires_at = Instant::now() + self.session_duration;
                return Some(session);
            }
        }
        None
    }

    pub fn destroy_session(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.sessions.retain(|_, session| session.expires_at > now);
    }
}

// Simple pseudo-random token generator using system time
fn rand_id() -> u128 {
    use std::time::SystemTime;
    let start = SystemTime::now();
    let since_epoch = start.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    since_epoch.as_nanos() ^ ((since_epoch.as_secs() as u128) << 64)
}