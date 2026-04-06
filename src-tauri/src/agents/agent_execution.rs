use serde::{Deserialize, Serialize};
use crate::agents::agent_intent::{AgentIntent, AgentExecutionLeaseRef};
use crate::authorities::aip_enforcer::{AipEnforcer, AipContext};
use crate::authorities::authority_protocol::{AuthorityActionClass};
use crate::authorities::authority_decision::AuthorityDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionRequest {
    pub intent: AgentIntent,
    pub lease_ref: AgentExecutionLeaseRef,
    pub target_scope: String, // e.g. "personal", "room"
    pub routing_lane: String,
}

pub struct AgentExecutionManager;

impl AgentExecutionManager {
    pub fn translate_intent_to_request(
        intent: AgentIntent,
        lease: AgentExecutionLeaseRef,
        scope: &str,
    ) -> Result<AgentExecutionRequest, String> {
        // AIP v2 Enforcement Gate
        let aip_ctx = AipContext {
            action_id: uuid::Uuid::new_v4(),
            action_type: AuthorityActionClass::ModelActivation, // Using ModelActivation as a proxy for agent activation
            identity_scope: format!("agent.execution.{}", scope),
            evidence_ref: Some(uuid::Uuid::new_v4()), // Simulated normalized evidence
            trust_chain: None, // Derived from orchestrator context
            target_resource: intent.agent_id.clone(),
            requested_by: "orchestrator".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let aip_result = AipEnforcer::enforce(aip_ctx);
        if !matches!(aip_result.decision, AuthorityDecision::Allow) {
            return Err(format!("AIP Enforcement Veto for Agent {}: {}", intent.agent_id, aip_result.reason));
        }

        Ok(AgentExecutionRequest {
            intent,
            lease_ref: lease,
            target_scope: scope.to_string(),
            routing_lane: "default".to_string(), // Shard-aware lane would be derived here
        })
    }
}
