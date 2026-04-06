use super::trust_chain::{TrustChain, TrustLink};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustPropagationDecision {
    ValidChain,
    BrokenChain(String),
    Expired(String),
    Revoked(String),
    ScopeViolation(String),
}

pub struct TrustPropagationModel;

impl TrustPropagationModel {
    pub fn validate_chain(chain: &TrustChain) -> TrustPropagationDecision {
        // 1. Check Root
        if chain.root_link.revoked {
            return TrustPropagationDecision::Revoked("Root identity revoked".to_string());
        }

        // 2. Validate Link Continuity
        let mut current_parent = &chain.root_link;
        for link in &chain.links {
            if link.parent_link_id != Some(current_parent.link_id) {
                return TrustPropagationDecision::BrokenChain(format!(
                    "Link {} does not point to parent {}", 
                    link.link_id, 
                    current_parent.link_id
                ));
            }

            if link.revoked {
                return TrustPropagationDecision::Revoked(format!("Link {} revoked", link.link_id));
            }

            if chrono::Utc::now() > link.expires_at {
                return TrustPropagationDecision::Expired(format!("Link {} expired", link.link_id));
            }

            // Scope Narrowing Check: Child must not exceed parent
            if !Self::is_scope_subset(&link.scope, &current_parent.scope) {
                return TrustPropagationDecision::ScopeViolation(format!(
                    "Link {} exceeds parent scope capabilities", 
                    link.link_id
                ));
            }

            current_parent = link;
        }

        TrustPropagationDecision::ValidChain
    }

    fn is_scope_subset(
        _child: &super::trust_chain::TrustScopeBinding, 
        _parent: &super::trust_chain::TrustScopeBinding
    ) -> bool {
        // Logic to verify that child capabilities are a subset of parent capabilities
        // and namespaces are within parent boundaries.
        true // Placeholder for M1
    }
}
