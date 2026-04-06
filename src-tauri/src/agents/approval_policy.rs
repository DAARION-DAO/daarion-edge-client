use serde::{Deserialize, Serialize};
use crate::agents::approval_proposal::ApprovalProposal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    NeedsReview,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicyResult {
    pub proposal_id: String,
    pub decision: ApprovalDecision,
    pub reason: String,
    pub evaluator_id: String,
}

pub struct ApprovalPolicyEvaluator;

impl ApprovalPolicyEvaluator {
    pub fn evaluate(proposal: &ApprovalProposal) -> ApprovalPolicyResult {
        // First-pass: Check scope compatibility
        if proposal.trust_scope != "NodeAdmin" && proposal.scope == crate::agents::approval_proposal::ApprovalScope::Node {
             return ApprovalPolicyResult {
                 proposal_id: proposal.proposal_id.clone(),
                 decision: ApprovalDecision::Rejected,
                 reason: "Agent lacks NodeAdmin trust scope for this proposal".to_string(),
                 evaluator_id: "local_evaluator".to_string(),
             };
        }

        // Check telemetry basis
        if proposal.telemetry_refs.is_empty() {
             return ApprovalPolicyResult {
                 proposal_id: proposal.proposal_id.clone(),
                 decision: ApprovalDecision::NeedsReview,
                 reason: "Insufficient telemetry basis for automated evaluation".to_string(),
                 evaluator_id: "local_evaluator".to_string(),
             };
        }

        ApprovalPolicyResult {
             proposal_id: proposal.proposal_id.clone(),
             decision: ApprovalDecision::Approved,
             reason: "Proposal meets safety constraints".to_string(),
             evaluator_id: "local_evaluator".to_string(),
        }
    }
}
