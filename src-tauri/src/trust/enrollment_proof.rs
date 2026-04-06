use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentProof {
    pub nonce: String,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub hardware_id: String,
    pub attestation_data: Option<Vec<u8>>,
}

impl EnrollmentProof {
    pub fn generate_proof(node_id: &str, private_key_ref: &str) -> Self {
        // Mocking signing logic for v1
        Self {
            nonce: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            signature: vec![0xDE, 0xAD, 0xBE, 0xEF], // Signed by private key
            hardware_id: format!("hw-{}", node_id),
            attestation_data: None,
        }
    }
}
