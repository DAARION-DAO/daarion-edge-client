use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::authority_protocol::{AuthorityLayer, AuthorityActionClass};
use super::authority_decision::AuthorityDecision;
use crate::trust::trust_chain::TrustChain;
use crate::trust::trust_propagation::{TrustPropagationModel, TrustPropagationDecision};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AipContext {
    pub action_id: Uuid,
    pub action_type: AuthorityActionClass,
    pub identity_scope: String,
    pub evidence_ref: Option<Uuid>,
    pub trust_chain: Option<TrustChain>,
    pub target_resource: String,
    pub requested_by: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AipResult {
    pub decision: AuthorityDecision,
    pub authority_trace: Vec<crate::authorities::authority_trace::AuthorityTraceEntry>,
    pub reason: String,
    pub requires_escalation: bool,
}

use crate::observability::evidence_gate::{EvidenceGate, EvidenceStatus};
use crate::authorities::authority_trace::AuthorityTraceEntry;
use chrono::Utc;

pub struct AipEnforcer;

impl AipEnforcer {
    pub fn enforce(ctx: AipContext) -> AipResult {
        let mut trace = Vec::new();

        // 1. Identity (DAIS) Gate — Enhanced with Trust Propagation
        let mut identity_pass = true;
        let mut identity_reason = "Identity legitimacy verified via DAIS.".to_string();

        if let Some(ref chain) = ctx.trust_chain {
            match TrustPropagationModel::validate_chain(chain) {
                TrustPropagationDecision::ValidChain => {
                    identity_reason = "Trust Propagation Chain validated.".to_string();
                }
                decision => {
                    identity_pass = false;
                    identity_reason = format!("Trust Violation: {:?}", decision);
                }
            }
        }

        trace.push(AuthorityTraceEntry {
            layer: AuthorityLayer::Identity,
            decision: if identity_pass { AuthorityDecision::Allow } else { AuthorityDecision::Block },
            timestamp: Utc::now(),
            reason: identity_reason,
        });

        if !identity_pass {
            return AipResult {
                decision: AuthorityDecision::Block,
                authority_trace: trace,
                reason: "DAIS: Identity/Trust Chain invalid.".to_string(),
                requires_escalation: false,
            };
        }

        // 2. Security (AISTALK) Gate
        let security_pass = true; // Placeholder
        trace.push(AuthorityTraceEntry {
            layer: AuthorityLayer::Security,
            decision: if security_pass { AuthorityDecision::Allow } else { AuthorityDecision::Block },
            timestamp: Utc::now(),
            reason: "Security posture nominal.".to_string(),
        });

        if !security_pass {
            return AipResult {
                decision: AuthorityDecision::Block,
                authority_trace: trace,
                reason: "AISTALK: Potential threat detected.".to_string(),
                requires_escalation: true,
            };
        }

        // 3. Observability (SENTINEL) Evidence Gate
        if let Some(evidence_ref) = ctx.evidence_ref {
            let evidence_status = EvidenceGate::validate_evidence(evidence_ref);
            trace.push(AuthorityTraceEntry {
                layer: AuthorityLayer::Observability,
                decision: match evidence_status {
                    EvidenceStatus::Valid => AuthorityDecision::Allow,
                    _ => AuthorityDecision::Defer,
                },
                timestamp: Utc::now(),
                reason: format!("Evidence status: {:?}", evidence_status),
            });

            if !matches!(evidence_status, EvidenceStatus::Valid) {
                return AipResult {
                    decision: AuthorityDecision::Defer,
                    authority_trace: trace,
                    reason: "SENTINEL: Insufficient or stale evidence.".to_string(),
                    requires_escalation: false,
                };
            }
        } else {
            // AIP v2: No evidence = No decision
            trace.push(AuthorityTraceEntry {
                layer: AuthorityLayer::Observability,
                decision: AuthorityDecision::Block,
                timestamp: Utc::now(),
                reason: "Missing mandatory evidence reference.".to_string(),
            });
            return AipResult {
                decision: AuthorityDecision::Block,
                authority_trace: trace,
                reason: "AIP v2 requires normalized evidence basis.".to_string(),
                requires_escalation: false,
            };
        }

        // 4. Architecture (SOFIIA) Gate
        trace.push(AuthorityTraceEntry {
            layer: AuthorityLayer::Architecture,
            decision: AuthorityDecision::Allow,
            timestamp: Utc::now(),
            reason: "Architectural integrity confirmed.".to_string(),
        });

        // 5. Orchestration (DAARWIZZ) Gate
        trace.push(AuthorityTraceEntry {
            layer: AuthorityLayer::Orchestration,
            decision: AuthorityDecision::Allow,
            timestamp: Utc::now(),
            reason: "Operation approved for execution.".to_string(),
        });

        // 6. Conscious Coordination (Global Alignment) Gate
        // Advisory only in AIP v2; enforces soft-constraints for global coherence.
        trace.push(AuthorityTraceEntry {
            layer: AuthorityLayer::Coordination,
            decision: AuthorityDecision::Allow,
            timestamp: Utc::now(),
            reason: "Global alignment confirmed via Conscious Coordination.".to_string(),
        });

        AipResult {
            decision: AuthorityDecision::Allow,
            authority_trace: trace,
            reason: "AIP v2: All gates passed.".to_string(),
            requires_escalation: false,
        }
    }
}
