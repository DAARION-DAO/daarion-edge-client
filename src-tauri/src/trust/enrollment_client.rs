use crate::trust::csr_submit::{CsrSubmissionRequest, CsrSubmissionResponse};
use crate::trust::enrollment_proof::EnrollmentProof;
use crate::trust::certificate_store::{CertificateStore, CertificateStoreRecord};

pub struct EnrollmentClient {
    pub node_id: String,
    pub region: String,
    pub district: String,
}

impl EnrollmentClient {
    pub async fn enroll(&self, store: &mut CertificateStore) -> Result<CsrSubmissionResponse, String> {
        // 1. Generate CSR (Mocked PEM generation)
        let csr_pem = format!("-----BEGIN CERTIFICATE REQUEST-----\nNODE:{}\n-----END CERTIFICATE REQUEST-----", self.node_id);
        
        // 2. Attach Proof
        let proof = EnrollmentProof::generate_proof(&self.node_id, "secure_identity_ref");

        // 3. Create Submission Request
        let request = CsrSubmissionRequest {
            node_id: self.node_id.clone(),
            public_key_der: vec![0, 1, 2, 3],
            csr_pem,
            region: self.region.clone(),
            district: self.district.clone(),
            tier: "Pro".to_string(),
            specializations: vec!["llm".to_string(), "vision".to_string()],
            capability_hash: "hash-v1".to_string(),
            enrollment_proof: bincode::serialize(&proof).unwrap_or_default(),
            app_version: "1.5.0".to_string(),
            created_at: chrono::Utc::now().timestamp_millis() as u64,
        };

        // 4. Submit to Issuer (Mocked Response)
        let response = CsrSubmissionResponse {
            certificate_pem: "-----BEGIN CERTIFICATE-----\nISSUED_IDENTITY\n-----END CERTIFICATE-----".to_string(),
            certificate_chain: vec!["-----BEGIN CERTIFICATE-----\nREGIONAL_CA\n-----END CERTIFICATE-----".to_string()],
            issuer_id: "issuer-region-1".to_string(),
            valid_from: chrono::Utc::now().timestamp_millis() as u64,
            valid_to: chrono::Utc::now().timestamp_millis() as u64 + 86400000 * 30, // 30 days
            registration_id: uuid::Uuid::new_v4().to_string(),
            session_scope_hints: vec!["heartbeat".to_string(), "nats".to_string()],
        };

        // 5. Store record
        store.save_certificate(CertificateStoreRecord {
            registration_id: response.registration_id.clone(),
            certificate_pem: response.certificate_pem.clone(),
            chain_pems: response.certificate_chain.clone(),
            issuer_id: response.issuer_id.clone(),
            region: self.region.clone(),
            valid_to: response.valid_to,
        });

        Ok(response)
    }
}
