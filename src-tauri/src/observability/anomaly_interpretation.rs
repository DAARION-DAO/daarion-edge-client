use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyInterpretation {
    pub anomaly_id: Uuid,
    pub evidence_ref: Uuid,
    pub anomaly_class: String, // e.g., "PerformanceDegradation", "SecurityThreat"
    pub interpretation_notes: String,
    pub impact_severity: f32,
}
