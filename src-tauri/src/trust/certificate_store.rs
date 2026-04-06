use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateStoreRecord {
    pub registration_id: String,
    pub certificate_pem: String,
    pub chain_pems: Vec<String>,
    pub issuer_id: String,
    pub region: String,
    pub valid_to: u64,
}

pub struct CertificateStore {
    // In a real app, this would be persistent storage/database
    pub records: HashMap<String, CertificateStoreRecord>,
}

impl CertificateStore {
    pub fn new() -> Self {
        Self { records: HashMap::new() }
    }

    pub fn save_certificate(&mut self, record: CertificateStoreRecord) {
        self.records.insert(record.registration_id.clone(), record);
    }

    pub fn get_valid_certificate(&self, registration_id: &str) -> Option<&CertificateStoreRecord> {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        self.records.get(registration_id).filter(|r| r.valid_to > now)
    }
}
