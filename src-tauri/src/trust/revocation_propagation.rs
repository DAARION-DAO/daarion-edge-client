use uuid::Uuid;
use super::trust_chain::TrustChain;

pub struct RevocationPropagation;

impl RevocationPropagation {
    /// Check if any link in the chain has been revoked.
    pub fn is_any_revoked(chain: &TrustChain, _revocation_registry: &Vec<Uuid>) -> bool {
        if chain.root_link.revoked {
            return true;
        }
        
        for link in &chain.links {
            if link.revoked {
                return true;
            }
        }
        
        false
    }
}
