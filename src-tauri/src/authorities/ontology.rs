use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthorityFieldType {
    /// Field of Legitimacy / Existence (DAIS)
    Identity,
    /// Field of Survival / Integrity (AISTALK)
    Security,
    /// Field of Perception / Truth (SENTINEL)
    Observability,
    /// Field of Structure / Form (SOFIIA)
    Architecture,
    /// Field of Action / Flow (DAARWIZZ)
    Orchestration,
    /// Field of Value / Meaning / Incentive (MELISSA)
    Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityFieldProfile {
    pub field_type: AuthorityFieldType,
    pub influence_scope: FieldScope,
    pub dependencies: Vec<AuthorityFieldType>,
    pub constraints: Vec<OntologicalConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldScope {
    Local,
    District,
    Regional,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OntologicalConstraint {
    /// No Action without Identity
    ExistencePrecedesAction,
    /// No Action without Safety
    IntegrityGatesExecution,
    /// No Action without Evidence
    RationalityRequiresPerception,
    /// No Structure without Architecture
    OrderRequiresForm,
    /// No Propagation without Value
    GrowthRequiresUtility,
}

impl AuthorityFieldType {
    pub fn name(&self) -> &'static str {
        match self {
            AuthorityFieldType::Identity => "Identity Field",
            AuthorityFieldType::Security => "Security Field",
            AuthorityFieldType::Observability => "Observability Field",
            AuthorityFieldType::Architecture => "Architecture Field",
            AuthorityFieldType::Orchestration => "Orchestration Field",
            AuthorityFieldType::Value => "Value Field",
        }
    }

    pub fn primary_authority(&self) -> &'static str {
        match self {
            AuthorityFieldType::Identity => "DAIS",
            AuthorityFieldType::Security => "AISTALK",
            AuthorityFieldType::Observability => "SENTINEL",
            AuthorityFieldType::Architecture => "SOFIIA",
            AuthorityFieldType::Orchestration => "DAARWIZZ",
            AuthorityFieldType::Value => "MELISSA",
        }
    }
}
