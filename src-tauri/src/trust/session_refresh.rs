use serde::{Deserialize, Serialize};
use crate::trust::session_derivation::SessionCredentialProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCredentialLease {
    pub profile: SessionCredentialProfile,
    pub acquired_at: u64,
    pub refresh_at: u64, // TS when renewal should start
}

pub struct SessionRefreshManager;

impl SessionRefreshManager {
    pub fn calculate_refresh_time(valid_from: u64, valid_to: u64) -> u64 {
        let ttl = valid_to.saturating_sub(valid_from);
        // Refresh at 80% of TTL
        valid_from + (ttl * 4 / 5)
    }
}
