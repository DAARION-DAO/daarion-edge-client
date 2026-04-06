use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorityLayer {
    Identity,      // Layer 0 (DAIS)
    Security,      // Layer 1 (AISTALK)
    Architecture,  // Layer 2 (SOFIIA)
    Orchestration, // Layer 3 (DAARWIZZ)
    Observability, // Layer 4 (SENTINEL)
    Value,         // Layer 5 (MELISSA)
    Coordination,  // Global alignment
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorityActionClass {
    ModelActivation,
    WorkerLease,
    TrustEnrollment,
    PolicyChange,
    DistrictSpecialization,
}
