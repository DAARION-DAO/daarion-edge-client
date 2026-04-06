pub mod authority_protocol;
pub mod authority_decision;
pub mod authority_conflict;
pub mod authority_escalation;
pub mod authority_trace;
pub mod aip_enforcer;
pub mod authority_conflict_resolver;
pub mod authority_embodiment;
pub mod ontology;

use authority_protocol::{AuthorityLayer, AuthorityActionClass};
use authority_decision::{AuthorityDecision, AuthorityLayerDecision};
use authority_trace::AuthorityTraceEntry;
use aip_enforcer::{AipContext, AipResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorityProtocolResult {
    Allowed,
    Blocked(String),
    Vetoed(AuthorityLayer, String),
    Escalated(String),
    RequiresHumanReview(String),
}

pub struct AuthorityInteractionProtocol;

impl AuthorityInteractionProtocol {
    /// Evaluate a multi-authority action context based on AIP v1 precedence.
    pub fn evaluate_action(
        _action: AuthorityActionClass,
        decisions: Vec<AuthorityLayerDecision>,
    ) -> AuthorityProtocolResult {
        // AIP v1 Precedence: 
        // 1. Identity (DAIS)
        // 2. Security (AISTALK)
        // 3. Architecture (SOFIIA)
        // 4. Orchestration (DAARWIZZ)
        // Observability (SENTINEL) provides evidence but is not a gating block in this flow.

        let mut current_result = AuthorityProtocolResult::Allowed;

        // Check Identity (DAIS) first
        if let Some(d) = decisions.iter().find(|d| d.authority_layer == AuthorityLayer::Identity) {
            match d.decision {
                AuthorityDecision::Block | AuthorityDecision::Veto => {
                    return AuthorityProtocolResult::Blocked(format!("Identity Invalidation: {}", d.reason_summary));
                }
                _ => {}
            }
        }

        // Check Security (AISTALK)
        if let Some(d) = decisions.iter().find(|d| d.authority_layer == AuthorityLayer::Security) {
            match d.decision {
                AuthorityDecision::Block | AuthorityDecision::Veto => {
                    return AuthorityProtocolResult::Vetoed(AuthorityLayer::Security, d.reason_summary.clone());
                }
                _ => {}
            }
        }

        // Check Architecture (SOFIIA)
        if let Some(d) = decisions.iter().find(|d| d.authority_layer == AuthorityLayer::Architecture) {
            match d.decision {
                AuthorityDecision::Veto => {
                    return AuthorityProtocolResult::Vetoed(AuthorityLayer::Architecture, d.reason_summary.clone());
                }
                AuthorityDecision::Escalate | AuthorityDecision::RequireReview => {
                    return AuthorityProtocolResult::RequiresHumanReview(d.reason_summary.clone());
                }
                _ => {}
            }
        }

        // Check Orchestration (DAARWIZZ)
        if let Some(d) = decisions.iter().find(|d| d.authority_layer == AuthorityLayer::Orchestration) {
            match d.decision {
                AuthorityDecision::Block => {
                    return AuthorityProtocolResult::Blocked(format!("Orchestration Rejection: {}", d.reason_summary));
                }
                _ => {}
            }
        }

        current_result
    }
}
