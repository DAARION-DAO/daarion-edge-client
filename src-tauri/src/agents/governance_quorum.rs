use std::collections::HashMap;
use crate::agents::governance_review::{GovernanceReview, GovernanceDecision};
use crate::agents::governance_role::GovernanceRole;

pub struct GovernanceQuorum;

impl GovernanceQuorum {
    pub fn compute_final_decision(reviews: &[GovernanceReview]) -> GovernanceDecision {
        if reviews.is_empty() {
            return GovernanceDecision::NeedsHumanReview;
        }

        // 1. Any Veto immediately overrides everything
        if reviews.iter().any(|r| r.decision == GovernanceDecision::Veto) {
            return GovernanceDecision::Veto;
        }

        // 2. Any Escalation also halts autonomous flow
        if reviews.iter().any(|r| r.decision == GovernanceDecision::Escalate) {
            return GovernanceDecision::Escalate;
        }

        // 3. Count approvals by role
        let mut approvals = HashMap::new();
        for review in reviews {
            if review.decision == GovernanceDecision::Approve {
                approvals.insert(review.governance_role.clone(), true);
            }
        }

        // 4. v1 Policy: Require at least Architecture and Security approvals
        let has_arch = approvals.contains_key(&GovernanceRole::Architecture);
        let has_security = approvals.contains_key(&GovernanceRole::Security);

        if has_arch && has_security {
            GovernanceDecision::Approve
        } else if reviews.iter().any(|r| r.decision == GovernanceDecision::Reject) {
            GovernanceDecision::Reject
        } else {
            GovernanceDecision::NeedsHumanReview
        }
    }
}
