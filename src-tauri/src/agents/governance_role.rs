use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GovernanceRole {
    Architecture, // CTO Role (Sofiia)
    Security,     // Trust and Scope Validation
    Operations,   // Infrastructure and Capacity
    Placement,    // Gravity and Saturation Optimization
    Compliance,   // Policy and Bounds Alignment
}
