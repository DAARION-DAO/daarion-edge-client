use serde::{Deserialize, Serialize};
use crate::models::session_shard::ShardDeriver;
use crate::authorities::aip_enforcer::{AipEnforcer, AipContext};
use crate::authorities::authority_protocol::{AuthorityActionClass};
use crate::authorities::authority_decision::AuthorityDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEventChannel {
    Stream,
    Meta,
    Done,
    Error,
}

impl SessionEventChannel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Stream => "stream",
            Self::Meta => "meta",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
}

pub struct SessionRouting;

impl SessionRouting {
    pub fn get_sharded_subject(
        region: &str,
        session_class: &str,
        session_id: &str,
        channel: SessionEventChannel,
        shard_count: u32,
    ) -> String {
        let shard_id = ShardDeriver::derive_shard(session_id, shard_count);
        let shard_name = ShardDeriver::get_shard_name(shard_id);
        
        format!(
            "sess.{}.{}.{}.{}.{}",
            region,
            session_class,
            shard_name,
            session_id,
            channel.as_str()
        )
    }

    pub fn get_fallback_subject(session_id: &str) -> String {
        format!("inference.stream.{}", session_id)
    }

    pub fn validate_session(session_id: &str, session_class: &str) -> Result<(), String> {
        // AIP v2 Enforcement Gate
        let aip_ctx = AipContext {
            action_id: uuid::Uuid::new_v4(),
            action_type: AuthorityActionClass::ModelActivation, // Proxy for session activation
            identity_scope: format!("session.routing.{}", session_class),
            evidence_ref: Some(uuid::Uuid::new_v4()), // Simulated normalized evidence
            trust_chain: None, // Derived from gateway/user session
            target_resource: session_id.to_string(),
            requested_by: "gateway".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let aip_result = AipEnforcer::enforce(aip_ctx);
        if !matches!(aip_result.decision, AuthorityDecision::Allow) {
            return Err(format!("AIP Enforcement Veto for Session {}: {}", session_id, aip_result.reason));
        }

        Ok(())
    }
}
