use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EvidenceFreshness {
    pub last_updated: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl EvidenceFreshness {
    pub fn is_stale(&self) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_updated).num_seconds();
        elapsed > self.ttl_seconds as i64
    }
}
